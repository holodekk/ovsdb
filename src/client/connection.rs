use async_trait::async_trait;

use futures::stream::StreamExt;
use serde_json::Value;
use tokio::{
    io::AsyncWriteExt,
    net::{TcpStream, UnixStream},
};
use tokio_util::codec::FramedRead;

use super::codec::JsonCodec;
use super::request::Request;

#[async_trait]
pub trait Connection {
    async fn next(&mut self) -> Option<Result<Value, std::io::Error>>;
    async fn send(&mut self, request: Request) -> Result<(), std::io::Error>;
}

pub struct TcpConnection {
    reader: FramedRead<tokio::net::tcp::OwnedReadHalf, JsonCodec>,
    writer: tokio::net::tcp::OwnedWriteHalf,
}

impl TcpConnection {
    pub fn new(stream: TcpStream) -> Self {
        let (read, writer) = stream.into_split();
        let reader = FramedRead::new(read, JsonCodec::new());
        Self { reader, writer }
    }
}

#[async_trait]
impl Connection for TcpConnection {
    async fn next(&mut self) -> Option<Result<Value, std::io::Error>> {
        self.reader.next().await
    }

    async fn send(&mut self, request: Request) -> Result<(), std::io::Error> {
        let data = serde_json::to_vec(&request)?;
        self.writer.write_all(&data).await
    }
}

pub struct UnixConnection {
    reader: FramedRead<tokio::net::unix::OwnedReadHalf, JsonCodec>,
    writer: tokio::net::unix::OwnedWriteHalf,
}

impl UnixConnection {
    pub fn new(stream: UnixStream) -> Self {
        let (read, writer) = stream.into_split();
        let reader = FramedRead::new(read, JsonCodec::new());
        Self { reader, writer }
    }
}

#[async_trait]
impl Connection for UnixConnection {
    async fn next(&mut self) -> Option<Result<Value, std::io::Error>> {
        self.reader.next().await
    }

    async fn send(&mut self, request: Request) -> Result<(), std::io::Error> {
        let data = serde_json::to_vec(&request)?;
        self.writer.write_all(&data).await
    }
}
