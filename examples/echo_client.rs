use clap::Parser;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;
use tracing::error;

static INC: AtomicU64 = AtomicU64::new(0);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = Opt::parse();

    tracing_subscriber::fmt()
        .with_max_level(tracing_subscriber::filter::LevelFilter::TRACE)
        .init();

    tokio::spawn(async move {
        loop {
            println!("TPS:{}", INC.swap(0, Ordering::AcqRel));
            tokio::time::sleep(Duration::from_secs(1)).await
        }
    });

    let joins = (0..opt.task)
        .map(|_| {
            let addr = opt.addr.clone();
            let local = opt.local.clone();
            tokio::spawn(async move { run(&addr, 60, &local).await })
        })
        .collect::<Vec<_>>();

    for join in joins {
        join.await??;
    }

    Ok(())
}

#[derive(Parser)]
#[clap(name = "test udp echo client")]
struct Opt {
    #[clap(long, value_parser, default_value = "127.0.0.1:8287")]
    addr: String,
    #[clap(long, value_parser, default_value_t = 20000)]
    task: u32,
    #[clap(long, value_parser, default_value = "0.0.0.0:0")]
    local: String,
}

async fn run(addr: &str, time: u64, local: &str) -> anyhow::Result<()> {
    let sender = Arc::new(UdpSocket::bind(local).await?);
    sender.connect(addr).await?;

    let reader = sender.clone();
    tokio::spawn(async move {
        let mut recv_buf = [0u8; 4096];
        loop {
            match reader.recv(&mut recv_buf[..]).await {
                Ok(size) => {
                    INC.fetch_add(1, Ordering::Release);
                    assert_eq!(&recv_buf[..size], b"hello!");
                    reader.send(&recv_buf[..size]).await.unwrap();
                }
                Err(err) => {
                    error!("error:{}", err);
                    break;
                }
            }
        }
    });

    let message = b"hello!";
    sender.send(message).await?;

    tokio::time::sleep(Duration::from_secs(time)).await;
    Ok(())
}
