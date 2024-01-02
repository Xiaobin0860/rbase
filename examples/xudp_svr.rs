use clap::Parser;
use rbase::{Peer, UdpServer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::parse();
    let addr = opt.bind;
    UdpServer::new(&addr, |peer, mut reader, _| async move {
        while let Some(Ok(data)) = reader.recv().await {
            peer.as_ref().send(&data).await?;
        }
        Ok(())
    })?
    .set_peer_timeout_sec(20)
    .start(())
    .await?;

    Ok(())
}

#[derive(Parser)]
#[clap(name = "rbase udp echo server")]
struct Opt {
    #[clap(long, value_parser, default_value = "0.0.0.0:8287")]
    bind: String,
}
