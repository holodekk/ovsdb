use std::collections::HashMap;
use std::path::Path;

use tokio::{
    net::UnixStream,
    sync::{mpsc, oneshot},
    task::JoinHandle,
};

mod connection;
pub use connection::*;

pub mod codec;
pub mod request;
use request::*;
pub mod response;

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
    #[error("Protocol error")]
    Codec(#[from] codec::Error),
}

pub trait Entity {
    fn table_name(&self) -> &'static str;
}

pub enum ClientRequest {
    Single {
        tx: oneshot::Sender<response::Response>,
        request: Request,
    },
    Monitor {
        tx: mpsc::Sender<response::Response>,
        request: Request,
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

    pub async fn execute(
        &self,
        method: Method,
        params: Option<Vec<crate::ovsdb::Value>>,
    ) -> Result<oneshot::Receiver<response::Response>, Error> {
        let (tx, rx) = oneshot::channel();
        let request = Request::new(method, params);

        if let Some(s) = &self.request_sender {
            s.send(ClientRequest::Single { tx, request }).await?;
        }

        Ok(rx)
    }

    pub async fn echo(&self, params: EchoParams) -> Result<Vec<String>, Error> {
        match self.execute(request::Method::Echo, Some(params.0)).await {
            Ok(rx) => match rx.await {
                Ok(res) => {
                    let p: Vec<String> = serde_json::from_value(res.result)?;
                    Ok(p)
                }
                Err(_err) => Err(Error::Unknown),
            },
            Err(err) => Err(err),
        }
    }

    pub async fn get_schema(&self, database: &str) -> Result<crate::schema::Schema, Error> {
        match self
            .execute(
                request::Method::GetSchema,
                Some(vec![crate::ovsdb::Value::from(database)]),
            )
            .await
        {
            Ok(rx) => match rx.await {
                Ok(res) => {
                    let s: crate::schema::Schema = serde_json::from_value(res.result)?;
                    Ok(s)
                }
                Err(_err) => Err(Error::Unknown),
            },
            Err(err) => Err(err),
        }
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
    let mut oneshot_channels: HashMap<uuid::Uuid, oneshot::Sender<response::Response>> =
        HashMap::new();
    let mut monitor_channels: HashMap<uuid::Uuid, mpsc::Sender<response::Response>> =
        HashMap::new();

    loop {
        tokio::select! {
            Some(msg) = requests.recv() => {
                match msg {
                    ClientRequest::Single { tx, request } => {
                        oneshot_channels.insert(request.id, tx);
                        conn.send(request).await?;
                    },
                    ClientRequest::Monitor { tx, request } => {
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
                let res: response::Response = serde_json::from_value(data?)?;
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
