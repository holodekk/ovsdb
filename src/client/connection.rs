use async_trait::async_trait;

use futures::stream::StreamExt;
use tokio::{
    io::AsyncWriteExt,
    net::{TcpStream, UnixStream},
};
use tokio_util::codec::FramedRead;

use crate::protocol::{codec, Request};

#[async_trait]
pub trait Connection {
    async fn next(&mut self) -> Option<Result<serde_json::Value, codec::Error>>;
    async fn send(&mut self, request: Request) -> Result<(), std::io::Error>;
    async fn shutdown(&mut self) -> Result<(), std::io::Error>;
}

pub struct TcpConnection {
    reader: FramedRead<tokio::net::tcp::OwnedReadHalf, codec::JsonCodec>,
    writer: tokio::net::tcp::OwnedWriteHalf,
}

impl TcpConnection {
    pub fn new(stream: TcpStream) -> Self {
        let (read, writer) = stream.into_split();
        let reader = FramedRead::new(read, codec::JsonCodec::new());
        Self { reader, writer }
    }
}

#[async_trait]
impl Connection for TcpConnection {
    async fn next(&mut self) -> Option<Result<serde_json::Value, codec::Error>> {
        self.reader.next().await
    }

    async fn send(&mut self, request: Request) -> Result<(), std::io::Error> {
        let data = serde_json::to_vec(&request)?;
        self.writer.write_all(&data).await
    }

    async fn shutdown(&mut self) -> Result<(), std::io::Error> {
        todo!()
    }
}

pub struct UnixConnection {
    reader: FramedRead<tokio::net::unix::OwnedReadHalf, codec::JsonCodec>,
    writer: tokio::net::unix::OwnedWriteHalf,
}

impl UnixConnection {
    pub fn new(stream: UnixStream) -> Self {
        let (read, writer) = stream.into_split();
        let reader = FramedRead::new(read, codec::JsonCodec::new());
        Self { reader, writer }
    }
}

#[async_trait]
impl Connection for UnixConnection {
    async fn next(&mut self) -> Option<Result<serde_json::Value, codec::Error>> {
        self.reader.next().await
    }

    async fn send(&mut self, request: Request) -> Result<(), std::io::Error> {
        let data = serde_json::to_vec(&request)?;
        println!("Sending {}", String::from_utf8(data.clone()).unwrap());
        self.writer.write_all(&data).await
    }

    async fn shutdown(&mut self) -> Result<(), std::io::Error> {
        self.writer.shutdown().await?;
        Ok(())
    }
}
