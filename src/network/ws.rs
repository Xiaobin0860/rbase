use crate::BaseError;
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use std::future::Future;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async_with_config, MaybeTlsStream, WebSocketStream};
use tracing::{error, trace};

pub use tokio_tungstenite::tungstenite::protocol::Message as WsMessage;

type WS = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub type WSocketSink = Arc<Mutex<SplitSink<WS, WsMessage>>>;
type WSocketStream = Arc<Mutex<SplitStream<WS>>>;

pub trait WsHandler {
    type Error: std::error::Error + Send + Sync + 'static;
    type Future: Future<Output = Result<Option<WsMessage>, Self::Error>> + Send + Sync;

    fn call(&self, request: WsMessage) -> Self::Future;
}

#[derive(Debug, Clone)]
/// one to one (connect)
pub struct WsClient {
    sink: WSocketSink,
    stream: WSocketStream,
}

impl WsClient {
    pub async fn new(addr: &str) -> Result<Self, BaseError> {
        let (ws, _) = connect_async_with_config(addr, None, true).await?;
        let (sink, stream) = ws.split();
        Ok(Self {
            sink: Arc::new(Mutex::new(sink)),
            stream: Arc::new(Mutex::new(stream)),
        })
    }

    pub fn get_ws(&self) -> WSocketSink {
        self.sink.clone()
    }

    pub async fn run<H>(&self, handler: &H)
    where
        H: WsHandler + Send + Sync + Clone + 'static,
        H::Error: std::fmt::Debug,
    {
        let mut stream = self.stream.lock().await;
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(msg) => {
                    let handler = handler.clone();
                    let sink = self.sink.clone();
                    tokio::spawn(async move {
                        match handler.call(msg).await {
                            Ok(Some(msg)) => {
                                let mut sink = sink.lock().await;
                                match sink.send(msg).await {
                                    Ok(_) => trace!("send RES ok"),
                                    Err(e) => error!("send RES error: {e:?}"),
                                }
                            }
                            Ok(None) => {}
                            Err(e) => error!("ws handler error: {:?}", e),
                        }
                    });
                }
                Err(e) => error!("ws error: {:?}", e),
            }
        }
    }

    pub async fn text(&self, msg: String) -> Result<(), BaseError> {
        let mut sink = self.sink.lock().await;
        sink.send(WsMessage::Text(msg)).await?;
        Ok(())
    }
}
