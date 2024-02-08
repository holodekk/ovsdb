use std::collections::HashMap;
use std::path::Path;

use serde::de::DeserializeOwned;
use tokio::{
    net::UnixStream,
    sync::{mpsc, oneshot},
    task::JoinHandle,
};

mod connection;
pub use connection::*;

pub mod generator;

use crate::protocol;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unknown Error")]
    Unknown,
    #[error("Not Connected")]
    NotConnected,
    #[error("Unexpected IO Error")]
    Io(#[from] std::io::Error),
    #[error("Serialization/deserialization Error")]
    Serde(#[from] serde_json::Error),
    #[error("Internal synchronization error")]
    Synchronization(#[from] mpsc::error::SendError<ClientRequest>),
    #[error("Internal synchronization error")]
    InternalSync(#[from] mpsc::error::SendError<ClientCommand>),
    #[error("Shutdown error")]
    Shutdown(#[from] tokio::task::JoinError),
    #[error("Tokio receive")]
    TokioReceive(#[from] tokio::sync::oneshot::error::RecvError),
    #[error("Protocol error")]
    Codec(#[from] protocol::codec::Error),
}

pub trait Entity {
    fn table_name() -> &'static str;
}

pub enum ClientRequest {
    Single {
        tx: oneshot::Sender<protocol::Response>,
        method: protocol::Method,
        params: protocol::Params,
    },
    Monitor {
        tx: mpsc::Sender<protocol::Response>,
        method: protocol::Method,
        params: protocol::Params,
    },
}

pub enum ClientCommand {
    Shutdown,
}

pub struct Client {
    pub request_sender: Option<mpsc::Sender<ClientRequest>>,
    pub command_sender: Option<mpsc::Sender<ClientCommand>>,
    pub handle: JoinHandle<Result<(), Error>>,
}

impl Client {
    pub fn new(
        request_sender: mpsc::Sender<ClientRequest>,
        command_sender: mpsc::Sender<ClientCommand>,
        handle: JoinHandle<Result<(), Error>>,
    ) -> Self {
        Self {
            request_sender: Some(request_sender),
            command_sender: Some(command_sender),
            handle,
        }
    }

    pub async fn start<T>(conn: T) -> Result<Self, Error>
    where
        T: Connection + Send + 'static,
    {
        let (requests_tx, requests_rx) = mpsc::channel(32);
        let (commands_tx, commands_rx) = mpsc::channel(32);

        let handle =
            { tokio::spawn(async move { client_main(requests_rx, commands_rx, conn).await }) };

        Ok(Client::new(requests_tx, commands_tx, handle))
    }

    pub async fn connect_unix(socket: &Path) -> Result<Client, Error> {
        let stream = UnixStream::connect(socket).await?;
        let conn = UnixConnection::new(stream);
        Client::start(conn).await
    }

    pub async fn stop(mut self) -> Result<(), Error> {
        if let Some(sender) = self.command_sender.take() {
            sender.send(ClientCommand::Shutdown).await?;
            drop(sender);
        };
        if let Some(sender) = self.request_sender.take() {
            drop(sender);
        }

        self.handle.await?
    }

    pub async fn execute<T>(
        &self,
        method: protocol::Method,
        params: protocol::Params,
    ) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        let (tx, rx) = oneshot::channel();

        match &self.request_sender {
            Some(s) => {
                s.send(ClientRequest::Single { tx, method, params }).await?;
                let res = rx.await?;
                let val: T = serde_json::from_value(res.result)?;
                Ok(val)
            }
            None => Err(Error::Unknown),
        }
    }

    pub async fn echo<P, I>(&self, args: P) -> Result<Vec<String>, Error>
    where
        P: IntoIterator<Item = I>,
        I: Into<String>,
    {
        let params = protocol::Params::Echo(args.into_iter().map(|v| v.into()).collect());
        self.execute(protocol::Method::Echo, params).await
    }

    pub async fn list_databases(&self) -> Result<Vec<String>, Error> {
        self.execute(
            protocol::Method::ListDatabases,
            protocol::Params::ListDatabases,
        )
        .await
    }

    pub async fn get_schema<S>(&self, database: S) -> Result<crate::schema::Schema, Error>
    where
        S: Into<String>,
    {
        self.execute(
            protocol::Method::GetSchema,
            protocol::Params::GetSchema(database.into()),
        )
        .await
    }

    pub async fn transact<S, T>(
        &self,
        database: S,
        operations: Vec<protocol::Operation>,
    ) -> Result<T, Error>
    where
        S: Into<String>,
        T: DeserializeOwned,
    {
        self.execute(
            protocol::Method::Transact,
            protocol::Params::Transact {
                database: database.into(),
                operations,
            },
        )
        .await
    }
}

async fn client_main<T>(
    mut requests: mpsc::Receiver<ClientRequest>,
    mut commands: mpsc::Receiver<ClientCommand>,
    mut conn: T,
) -> Result<(), Error>
where
    T: Connection,
{
    let mut oneshot_channels: HashMap<uuid::Uuid, oneshot::Sender<protocol::Response>> =
        HashMap::new();
    let mut monitor_channels: HashMap<uuid::Uuid, mpsc::Sender<protocol::Response>> =
        HashMap::new();

    loop {
        tokio::select! {
            Some(msg) = requests.recv() => {
                match msg {
                    ClientRequest::Single { tx, method, params } => {
                        let request = protocol::Request::new(method, params);
                        oneshot_channels.insert(request.id, tx);
                        conn.send(request).await?;
                    },
                    ClientRequest::Monitor { tx, method, params } => {
                        let request = protocol::Request::new(method, params);
                        monitor_channels.insert(request.id, tx);
                        conn.send(request).await?;
                    }
                }
            },
            Some(cmd) = commands.recv() => {
                match cmd {
                    ClientCommand::Shutdown => {
                        conn.shutdown().await?;
                    }
                }
            }
            Some(data) = conn.next() => {
                println!("Received: {:#?}", data);
                let res: protocol::Response = serde_json::from_value(data?)?;
                println!("result: {}", res.result);
                if let Some(tx) = oneshot_channels.remove(&res.id) {
                    let _ = tx.send(res);
                } else if let Some(tx) = monitor_channels.get(&res.id) {
                    let _ = tx.send(res).await;
                }
            },
            else => {
                break;
            }
        }
    }

    Ok(())
}
