use std::future::Future;

const PACKET_SIZE: usize = 1152;

pub mod error;
pub mod time;

mod rd;
pub use rd::*;
mod sys;
pub use sys::*;
mod storage;
pub use storage::*;
mod network;
pub use network::*;
mod shared;
pub use shared::{value::*, *};
mod id;
pub use id::*;

pub type Bytes = Vec<u8>;

pub use error::BaseError;
pub use time::timestamp_sec;

pub trait Handler {
    type Error: std::error::Error + Send + Sync + 'static;
    type Future: Future<Output = Result<Option<Bytes>, Self::Error>> + Send + Sync;

    fn call(&mut self, request: Bytes) -> Self::Future;

    fn exit(&self) -> bool {
        false
    }
}
