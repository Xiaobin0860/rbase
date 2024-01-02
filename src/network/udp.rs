//! UDP client and server

use crate::{timestamp_sec, Peer, PACKET_SIZE};
use async_trait::async_trait;
use dashmap::DashMap;
use net2::UdpBuilder;
use std::convert::TryFrom;
use std::error::Error;
use std::fmt::Display;
use std::future::Future;
use std::io::{self, ErrorKind};
use std::marker::PhantomData;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tracing::{debug, error, info, trace, warn};

#[derive(Debug, Clone)]
/// one to one (connect)
pub struct UdpClient {
    pub addr: String,
    pub sock: Arc<UdpSocket>,
}

impl UdpClient {
    pub async fn new(addr: &str) -> io::Result<Self> {
        let sock = Arc::new(UdpSocket::bind("0.0.0.0:0").await?);
        sock.connect(addr).await?;
        Ok(Self {
            addr: addr.to_string(),
            sock,
        })
    }

    pub async fn run<H>(&self, handler: &mut H)
    where
        H: crate::Handler + Send + Clone + 'static,
        H::Error: std::fmt::Debug,
    {
        let mut buf = [0; crate::PACKET_SIZE];
        loop {
            if handler.exit() {
                break;
            }
            if let Err(e) = self.sock.readable().await {
                error!("readable error: {e:?}");
                continue;
            }
            match self.sock.try_recv(&mut buf) {
                Ok(n) => {
                    trace!("recv REQ {:?}", &buf[..n]);
                    let data = buf[..n].to_vec();
                    let sock = self.sock.clone();
                    let mut handler = handler.clone();
                    tokio::spawn(async move {
                        match handler.call(data).await {
                            Ok(Some(data)) => match sock.send(&data).await {
                                Ok(n) => trace!("send RES {:?}", &data[..n]),
                                Err(e) => error!("send RES error: {e:?}"),
                            },
                            Ok(None) => {}
                            Err(e) => {
                                error!("handler error: {e:?}");
                            }
                        }
                    });
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                Err(e) => {
                    warn!("try_recv error {e:?}");
                }
            }
        }
        info!("udp client {:?} exit", self.sock.local_addr());
    }

    pub async fn send(&self, data: &[u8]) -> io::Result<usize> {
        self.sock.send(data).await
    }
}

pub type UDPPeer = Arc<UdpPeer>;
pub type UdpSender = UnboundedSender<io::Result<Vec<u8>>>;
pub type UdpReader = UnboundedReceiver<io::Result<Vec<u8>>>;

/// UDP Peer
/// each address+port is equal to one UDP peer
pub struct UdpPeer {
    pub socket_id: usize,
    pub udp_sock: Arc<UdpSocket>,
    pub addr: SocketAddr,
    sender: UdpSender,
    last_read_time: AtomicI64,
}

impl Display for UdpPeer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.socket_id, self.addr)
    }
}

impl Drop for UdpPeer {
    fn drop(&mut self) {
        trace!(
            "udp_listen socket:{} udp peer:{} drop",
            self.socket_id,
            self.addr
        )
    }
}

impl UdpPeer {
    #[inline]
    pub fn new(
        socket_id: usize,
        udp_sock: Arc<UdpSocket>,
        addr: SocketAddr,
    ) -> (UDPPeer, UdpReader) {
        let (tx, rx) = unbounded_channel();
        (
            Arc::new(Self {
                socket_id,
                udp_sock,
                addr,
                sender: tx,
                last_read_time: AtomicI64::new(timestamp_sec()),
            }),
            rx,
        )
    }

    /// get last recv sec
    #[inline]
    pub(crate) fn get_last_recv_sec(&self) -> i64 {
        self.last_read_time.load(Ordering::Acquire)
    }

    /// push data to read tx
    #[inline]
    pub(crate) fn push_data(&self, buf: Vec<u8>) -> io::Result<()> {
        if let Err(err) = self.sender.send(Ok(buf)) {
            Err(io::Error::new(ErrorKind::Other, err))
        } else {
            Ok(())
        }
    }

    /// push data to read tx and update instant
    #[inline]
    pub(crate) async fn push_data_and_update_instant(&self, buf: Vec<u8>) -> io::Result<()> {
        self.last_read_time
            .store(timestamp_sec(), Ordering::Release);
        self.push_data(buf)
    }

    /// get socket id
    #[inline]
    pub fn get_socket_id(&self) -> usize {
        self.socket_id
    }

    #[inline]
    pub fn get_ipv4(&self) -> u32 {
        match self.addr {
            SocketAddr::V4(addr) => u32::from_be_bytes(addr.ip().octets()),
            _ => 0,
        }
    }

    #[inline]
    pub fn close(&self) {
        if let Err(err) = self.sender.send(Err(io::Error::new(
            ErrorKind::TimedOut,
            "udp peer need close",
        ))) {
            error!("send timeout to udp peer:{} error:{err}", self.get_addr());
        }
    }
}

#[async_trait]
impl Peer for UdpPeer {
    fn get_addr(&self) -> &SocketAddr {
        &self.addr
    }

    async fn send(&self, buf: &[u8]) -> io::Result<usize> {
        self.udp_sock.send_to(buf, &self.addr).await
    }
}

/// UDP Context
/// each bind will create a
pub struct UdpContext {
    pub id: usize,
    recv: Arc<UdpSocket>,
    pub peers: DashMap<SocketAddr, UDPPeer>,
}

/// UDP Server listen
pub struct UdpServer<I, T> {
    udp_contexts: Vec<Arc<UdpContext>>,
    input: Arc<I>,
    _ph: PhantomData<T>,
    clean_sec: Option<u64>,
}

impl<I, R, T> UdpServer<I, T>
where
    I: Fn(UDPPeer, UdpReader, T) -> R + Send + Sync + 'static,
    R: Future<Output = Result<(), Box<dyn Error>>> + Send + 'static,
    T: Sync + Send + Clone + 'static,
{
    /// new udp server
    pub fn new<A: ToSocketAddrs>(addr: A, input: I) -> io::Result<Self> {
        let udp_list = create_udp_socket_list(&addr, get_cpu_count())?;
        let udp_contexts = udp_list
            .into_iter()
            .enumerate()
            .map(|(id, socket)| {
                Arc::new(UdpContext {
                    id,
                    recv: Arc::new(socket),
                    peers: Default::default(),
                })
            })
            .collect();
        Ok(UdpServer {
            udp_contexts,
            input: Arc::new(input),
            _ph: Default::default(),
            clean_sec: None,
        })
    }

    /// set how long the packet is not obtained and close the udp peer
    #[inline]
    pub fn set_peer_timeout_sec(mut self, sec: u64) -> UdpServer<I, T> {
        assert!(sec > 0);
        self.clean_sec = Some(sec);
        self
    }

    /// start server
    #[inline]
    pub async fn start(&self, inner: T) -> io::Result<()> {
        let need_check_timeout = {
            if let Some(clean_sec) = self.clean_sec {
                let clean_sec = clean_sec as i64;
                let contexts = self.udp_contexts.clone();
                tokio::spawn(async move {
                    loop {
                        let current = chrono::Utc::now().timestamp();
                        for context in contexts.iter() {
                            context.peers.iter().for_each(|peer| {
                                let peer = peer.value();
                                if current - peer.get_last_recv_sec() > clean_sec {
                                    peer.close();
                                }
                            });
                        }
                        tokio::time::sleep(Duration::from_secs(5)).await
                    }
                });
                true
            } else {
                false
            }
        };

        let (tx, mut rx) = unbounded_channel();
        for (index, udp_listen) in self.udp_contexts.iter().enumerate() {
            let create_peer_tx = tx.clone();
            let udp_context = udp_listen.clone();
            tokio::spawn(async move {
                debug!("start udp listen:{index}");
                let mut buff = [0; PACKET_SIZE];
                loop {
                    match udp_context.recv.recv_from(&mut buff).await {
                        Ok((size, addr)) => {
                            let peer = {
                                udp_context
                                    .peers
                                    .entry(addr)
                                    .or_insert_with(|| {
                                        let (peer, reader) =
                                            UdpPeer::new(index, udp_context.recv.clone(), addr);
                                        trace!("create udp listen:{index} udp peer:{addr}");
                                        if let Err(err) =
                                            create_peer_tx.send((peer.clone(), reader, index, addr))
                                        {
                                            panic!("create_peer_tx err:{}", err);
                                        }
                                        peer
                                    })
                                    .clone()
                            };

                            if need_check_timeout {
                                if let Err(err) = peer
                                    .push_data_and_update_instant(buff[..size].to_vec())
                                    .await
                                {
                                    error!("peer push data and update instant is error:{err}");
                                }
                            } else if let Err(err) = peer.push_data(buff[..size].to_vec()) {
                                error!("peer push data is error:{err}");
                            }
                        }
                        Err(err) => {
                            trace!("udp:{index} recv_from error:{err}");
                        }
                    }
                }
            });
        }
        drop(tx);

        while let Some((peer, reader, index, addr)) = rx.recv().await {
            let inner = inner.clone();
            let input_fn = self.input.clone();
            let context = self
                .udp_contexts
                .get(index)
                .expect("not found context")
                .clone();
            tokio::spawn(async move {
                if let Err(err) = (input_fn)(peer, reader, inner).await {
                    error!("udp input error:{err}")
                }
                context.peers.remove(&addr);
            });
        }
        Ok(())
    }
}

///Create udp socket for windows
#[cfg(target_os = "windows")]
fn make_udp_client(addr: SocketAddr) -> io::Result<std::net::UdpSocket> {
    if addr.is_ipv4() {
        Ok(UdpBuilder::new_v4()?.reuse_address(true)?.bind(addr)?)
    } else if addr.is_ipv6() {
        Ok(UdpBuilder::new_v6()?.reuse_address(true)?.bind(addr)?)
    } else {
        Err(io::Error::new(io::ErrorKind::Other, "not address AF_INET"))
    }
}

///It is used to create udp sockets for non-windows. The difference from windows is that reuse_port
#[cfg(not(target_os = "windows"))]
fn make_udp_client(addr: SocketAddr) -> io::Result<std::net::UdpSocket> {
    use net2::unix::UnixUdpBuilderExt;
    if addr.is_ipv4() {
        Ok(UdpBuilder::new_v4()?
            .reuse_address(true)?
            .reuse_port(true)?
            .bind(addr)?)
    } else if addr.is_ipv6() {
        Ok(UdpBuilder::new_v6()?
            .reuse_address(true)?
            .reuse_port(true)?
            .bind(addr)?)
    } else {
        Err(io::Error::new(io::ErrorKind::Other, "not address AF_INET"))
    }
}

///Create a udp socket and set the buffer size
fn create_udp_socket<A: ToSocketAddrs>(addr: &A) -> io::Result<std::net::UdpSocket> {
    let addr = {
        let mut addrs = addr.to_socket_addrs()?;
        let addr = match addrs.next() {
            Some(addr) => addr,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "no socket addresses could be resolved",
                ))
            }
        };
        if addrs.next().is_none() {
            Ok(addr)
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "more than one address resolved",
            ))
        }
    };
    let res = make_udp_client(addr?)?;
    Ok(res)
}

/// From std socket create tokio udp socket
fn create_async_udp_socket<A: ToSocketAddrs>(addr: &A) -> io::Result<UdpSocket> {
    let std_sock = create_udp_socket(&addr)?;
    std_sock.set_nonblocking(true)?;
    let sock = UdpSocket::try_from(std_sock)?;
    Ok(sock)
}

/// create tokio UDP socket list
/// listen_count indicates how many UDP SOCKETS to listen
fn create_udp_socket_list<A: ToSocketAddrs>(
    addr: &A,
    listen_count: usize,
) -> io::Result<Vec<UdpSocket>> {
    debug!("cpus:{listen_count}");
    let mut listens = Vec::with_capacity(listen_count);
    for _ in 0..listen_count {
        let sock = create_async_udp_socket(addr)?;
        listens.push(sock);
    }
    Ok(listens)
}

#[cfg(not(target_os = "windows"))]
fn get_cpu_count() -> usize {
    num_cpus::get()
}

#[cfg(target_os = "windows")]
fn get_cpu_count() -> usize {
    1
}
