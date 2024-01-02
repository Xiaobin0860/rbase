use std::net::SocketAddr;

use async_trait::async_trait;

#[async_trait]
pub trait Peer {
    async fn send(&self, buf: &[u8]) -> std::io::Result<usize>;

    fn get_addr(&self) -> &SocketAddr;
}
