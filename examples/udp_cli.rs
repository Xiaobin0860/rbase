use std::{
    io::{self, stdin},
    sync::Arc,
};
use tokio::{net::UdpSocket, task::JoinHandle};

fn get_stdin_data() -> Result<String, Box<dyn std::error::Error>> {
    let mut buf = String::new();
    stdin().read_line(&mut buf)?;
    Ok(buf)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sock = Arc::new(UdpSocket::bind("0.0.0.0:0").await?);
    let remote_addr = "127.0.0.1:8287";
    sock.connect(remote_addr).await?;
    println!("connected to {remote_addr}");

    let reader = sock.clone();
    let reader_handle: JoinHandle<Result<(), _>> = tokio::spawn(async move {
        loop {
            reader.readable().await?;
            let mut buf = [0; 1024];
            match reader.try_recv(&mut buf) {
                Ok(n) => {
                    println!("GOT {:?}", &buf[..n]);
                    continue;
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    println!("WOULD BLOCK");
                    continue;
                }
                Err(e) => {
                    println!("ERR {:?}", e);
                    return Err(e);
                }
            }
        }
    });

    loop {
        let data = get_stdin_data()?;
        if data.starts_with("exit") {
            reader_handle.abort();
            break;
        }
        println!("try sending {data:?}");
        match sock.send(data.as_bytes()).await {
            Ok(len) => println!("{len} bytes sent, data={data:?}"),
            Err(e) => println!("failed to send data: {e}"),
        }
    }

    Ok(())
}
