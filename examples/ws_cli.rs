use std::io::stdin;

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{client_async, tungstenite::Message};

fn get_stdin_data() -> Result<String, Box<dyn std::error::Error>> {
    let mut buf = String::new();
    stdin().read_line(&mut buf)?;
    Ok(buf)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tcp = TcpStream::connect("127.0.0.1:8288")
        .await
        .expect("Failed to connect");
    let url = url::Url::parse("ws://127.0.0.1:8288/").unwrap();
    let (stream, _) = client_async(url, tcp)
        .await
        .expect("Client failed to connect");
    let (mut tx, mut rx) = stream.split();

    let recv_handle = tokio::spawn(async move {
        while let Some(Ok(msg)) = rx.next().await {
            println!("Received: {}", msg);
        }
    });

    loop {
        let data = get_stdin_data()?;
        if data.starts_with("exit") {
            break;
        }
        println!("Sending {data}");
        tx.send(Message::Text(data))
            .await
            .expect("Failed to send message");
    }

    tx.close().await.expect("Failed to close");

    recv_handle.await?;

    Ok(())
}
