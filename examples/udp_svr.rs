use clap::Parser;
use std::io;
use tokio::net::UdpSocket;

#[tokio::main]
async fn main() -> io::Result<()> {
    let opt = Opt::parse();
    let addr = opt.bind;
    let sock = UdpSocket::bind(&addr).await?;
    loop {
        //println!("waiting on {addr} ...");
        let mut buf = [0; 1024];
        let (len, addr) = sock.recv_from(&mut buf).await?;
        //println!("{len} bytes received from {addr:?}");

        let _len = sock.send_to(&buf[..len], addr).await?;
        //println!("{len} bytes sent");
    }
}

#[derive(Parser)]
#[clap(name = "tokio udp echo server")]
struct Opt {
    #[clap(long, value_parser, default_value = "0.0.0.0:8287")]
    bind: String,
}
