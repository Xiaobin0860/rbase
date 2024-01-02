use futures_util::Future;
use rbase::{WsClient, WsHandler, WsMessage};
use std::sync::atomic::AtomicI32;
use std::sync::Arc;
use std::{io::stdin, pin::Pin};

fn get_stdin_data() -> Result<String, Box<dyn std::error::Error>> {
    let mut buf = String::new();
    stdin().read_line(&mut buf)?;
    Ok(buf)
}

#[derive(Debug, Default, Clone)]
struct EchoClient {
    count: Arc<AtomicI32>,
}

impl EchoClient {
    fn new() -> Self {
        Default::default()
    }
}

impl WsHandler for EchoClient {
    type Error = std::io::Error;
    type Future =
        Pin<Box<dyn Future<Output = Result<Option<WsMessage>, Self::Error>> + Send + Sync>>;

    fn call(&self, request: WsMessage) -> Self::Future {
        let count = self.count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Box::pin(async move {
            println!("recv{count}: {request:?}");
            Ok(None)
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing_subscriber::filter::LevelFilter::TRACE)
        .init();

    let client = WsClient::new("ws://127.0.0.1:8288").await?;

    let handler = EchoClient::new();
    println!("run client {handler:?}");

    let reader = client.clone();
    tokio::spawn(async move {
        reader.run(&handler).await;
    });

    loop {
        println!("input: ");
        let data = get_stdin_data()?;
        if data.starts_with("exit") {
            break;
        }
        print!("send: {data}");
        client.text(data).await?;
    }

    Ok(())
}
