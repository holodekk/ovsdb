use std::collections::HashMap;
use std::path::Path;

use futures::{stream::StreamExt, SinkExt};
use ovsdb::protocol;
use serde::de::DeserializeOwned;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
    sync::{mpsc, oneshot},
    task::JoinHandle,
};
use tokio_util::codec::Framed;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to deliver command")]
    CommandSend(#[from] mpsc::error::SendError<ClientCommand>),
    #[error("Failed to deliver request")]
    RequestSend(#[from] mpsc::error::SendError<ClientRequest>),
    #[error("Empty result received from server")]
    EmptyResult,
    #[error("Failed to receive response to request")]
    Response(#[from] oneshot::error::RecvError),
    #[error("OVSDB error")]
    OVSDB(#[from] ovsdb::Error),
    #[error("Failed to shutdown client")]
    Shutdown(#[from] tokio::task::JoinError),
    #[error("IO Error")]
    Io(#[from] std::io::Error),
    #[error("Client thread not active")]
    NotRunning,
}

type Result<T> = std::result::Result<T, Error>;

pub trait Entity {
    fn table_name() -> &'static str;
}

pub struct ClientRequest {
    tx: oneshot::Sender<protocol::Response>,
    method: protocol::Method,
    params: protocol::Params,
}

pub enum ClientCommand {
    Shutdown,
}

pub struct Client {
    pub request_sender: Option<mpsc::Sender<ClientRequest>>,
    pub command_sender: Option<mpsc::Sender<ClientCommand>>,
    pub handle: JoinHandle<Result<()>>,
}

impl Client {
    pub fn new(
        request_sender: mpsc::Sender<ClientRequest>,
        command_sender: mpsc::Sender<ClientCommand>,
        handle: JoinHandle<Result<()>>,
    ) -> Self {
        Self {
            request_sender: Some(request_sender),
            command_sender: Some(command_sender),
            handle,
        }
    }

    pub async fn start<T>(stream: T) -> Result<Self>
    where
        T: AsyncWriteExt + AsyncReadExt + Send + 'static,
    {
        let (requests_tx, requests_rx) = mpsc::channel(32);
        let (commands_tx, commands_rx) = mpsc::channel(32);

        let handle =
            { tokio::spawn(async move { client_main(requests_rx, commands_rx, stream).await }) };

        Ok(Client::new(requests_tx, commands_tx, handle))
    }

    pub async fn connect_unix(socket: &Path) -> Result<Client> {
        let stream = UnixStream::connect(socket).await?;
        Client::start(stream).await
    }

    pub async fn stop(mut self) -> Result<()> {
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
    ) -> Result<Option<T>>
    where
        T: DeserializeOwned,
    {
        let (tx, rx) = oneshot::channel();

        match &self.request_sender {
            Some(s) => {
                s.send(ClientRequest { tx, method, params }).await?;
                let res = rx.await?;
                if let Some(r) = res.result {
                    let val: T = protocol::decode(r)?;
                    Ok(Some(val))
                } else {
                    Ok(None)
                }
            }
            None => Err(Error::NotRunning),
        }
    }

    pub async fn echo<P, I>(&self, args: P) -> Result<Vec<String>>
    where
        P: IntoIterator<Item = I>,
        I: Into<String>,
    {
        let params = protocol::Params::Echo(args.into_iter().map(|v| v.into()).collect());
        match self.execute(protocol::Method::Echo, params).await? {
            Some(data) => Ok(data),
            None => Err(Error::EmptyResult),
        }
    }

    pub async fn list_databases(&self) -> Result<Vec<String>> {
        match self
            .execute(
                protocol::Method::ListDatabases,
                protocol::Params::ListDatabases,
            )
            .await?
        {
            Some(data) => Ok(data),
            None => Err(Error::EmptyResult),
        }
    }

    pub async fn get_schema<S>(&self, database: S) -> Result<ovsdb::schema::Schema>
    where
        S: Into<String>,
    {
        match self
            .execute(
                protocol::Method::GetSchema,
                protocol::Params::GetSchema(database.into()),
            )
            .await?
        {
            Some(data) => Ok(data),
            None => Err(Error::EmptyResult),
        }
    }

    pub async fn transact<S, T>(
        &self,
        database: S,
        operations: Vec<protocol::Operation>,
    ) -> Result<T>
    where
        S: Into<String>,
        T: DeserializeOwned,
    {
        match self
            .execute(
                protocol::Method::Transact,
                protocol::Params::Transact(protocol::TransactParams::new(database, operations)),
            )
            .await?
        {
            Some(data) => Ok(data),
            None => Err(Error::EmptyResult),
        }
    }
}

async fn client_main<T>(
    mut requests: mpsc::Receiver<ClientRequest>,
    mut commands: mpsc::Receiver<ClientCommand>,
    stream: T,
) -> Result<()>
where
    T: AsyncReadExt + AsyncWriteExt,
{
    let (mut writer, mut reader) = Framed::new(stream, protocol::Codec::new()).split();
    let mut channels: HashMap<protocol::Uuid, oneshot::Sender<protocol::Response>> = HashMap::new();

    loop {
        tokio::select! {
            Some(req) = requests.recv() => {
                let request = protocol::Request::new(req.method, req.params);
                if let Some(id) = &request.id {
                    channels.insert(id.clone(), req.tx);
                }
                writer.send(request.into()).await?;
            },
            Some(cmd) = commands.recv() => {
                match cmd {
                    ClientCommand::Shutdown => {
                        writer.close().await?;
                        // todo!()
                        // writer.
                        // writer.shutdown().await?;
                    }
                }
            }
            Some(msg) = reader.next() => {
                match msg {
                    Ok(protocol::Message::Response(res)) => {
                        if let Some(id) = &res.id {
                            if let Some(tx) = channels.remove(id) {
                                let _ = tx.send(res);
                            }
                        }
                    },
                    Ok(protocol::Message::Request(_req)) => {
                        todo!();
                    },
                    Err(_e) => todo!()
                }
            },
            else => {
                break;
            }
        }
    }

    Ok(())
}
