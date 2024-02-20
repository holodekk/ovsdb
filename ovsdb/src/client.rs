//! TCP/Unix socket based OVSDB client.
use std::collections::HashMap;
use std::path::Path;

use futures::{stream::StreamExt, SinkExt};
use serde::de::DeserializeOwned;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
    sync::{
        mpsc::{self, error::SendError},
        oneshot::{self, error::RecvError},
    },
    task::JoinHandle,
};
use tokio_util::codec::Framed;

use crate::protocol::{
    method::{
        EchoParams, EchoResult, GetSchemaParams, ListDbsResult, Method, Operation, TransactParams,
    },
    Request,
};

use super::{protocol, schema::Schema};

/// Internal synchronization failure
#[derive(Debug)]
pub struct SynchronizationError(String);

impl std::fmt::Display for SynchronizationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Synchronication failure: {}", self.0)
    }
}

impl std::error::Error for SynchronizationError {}

impl From<SendError<ClientCommand>> for SynchronizationError {
    fn from(err: SendError<ClientCommand>) -> Self {
        Self(err.to_string())
    }
}

impl From<SendError<ClientRequest>> for SynchronizationError {
    fn from(err: SendError<ClientRequest>) -> Self {
        Self(err.to_string())
    }
}

impl From<RecvError> for SynchronizationError {
    fn from(err: RecvError) -> Self {
        Self(err.to_string())
    }
}

/// The error type for operations performed by the [Client].
#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    /// An internal error occurred during synchronization.
    #[error("Failed to deliver command")]
    Internal(#[from] SynchronizationError),
    /// An error occurred trying to establish a connection with the OVSDB server.
    #[error("Failed to establish connection with the server")]
    ConnectionFailed(#[source] std::io::Error),
    /// An error occurred while trying to shutdown the client main loop.
    #[error("Failed to shutdown client")]
    ShutdownError(#[from] tokio::task::JoinError),
    /// A client method was executed, but the client is not connected to OVSDB.
    #[error("Client thread not active")]
    NotRunning,
    /// A response was received from the OVSDB server that could not be processed.
    #[error("Unexpected result received in response object")]
    UnexpectedResult,
    /// An error was encountered while processing send/receive with the OVSDB server.
    #[error("An error occurred when communicating with the server")]
    CommunicationFailure(#[from] protocol::CodecError),
    /// A low-level OVSDB error was encountered.
    #[error("OVSDB error")]
    OvsdbError(#[from] crate::Error),
}

#[derive(Debug)]
struct ClientRequest {
    tx: oneshot::Sender<protocol::Response>,
    request: Request,
}

#[derive(Clone, Copy, Debug)]
enum ClientCommand {
    Shutdown,
}

/// An OVSDB client, used to interact with an OVSDB database server.
///
/// The client is a thin wrapper around the various methods available in the OVSDB protocol.
/// Instantiating a client is done through one of the `connect_` methods.
///
/// # Examples
///
/// ```rust,no_run
/// use ovsdb::client::Client;
///
/// let client = Client::connect_unix(Path::new("/var/run/openvswitch/db.sock"))
///     .await
///     .unwrap();
/// ```
#[derive(Debug)]
pub struct Client {
    request_sender: Option<mpsc::Sender<ClientRequest>>,
    command_sender: Option<mpsc::Sender<ClientCommand>>,
    handle: JoinHandle<Result<(), ClientError>>,
}

impl Client {
    fn new(
        request_sender: mpsc::Sender<ClientRequest>,
        command_sender: mpsc::Sender<ClientCommand>,
        handle: JoinHandle<Result<(), ClientError>>,
    ) -> Self {
        Self {
            request_sender: Some(request_sender),
            command_sender: Some(command_sender),
            handle,
        }
    }

    async fn start<T>(stream: T) -> Result<Self, ClientError>
    where
        T: AsyncWriteExt + AsyncReadExt + Send + 'static,
    {
        let (requests_tx, requests_rx) = mpsc::channel(32);
        let (commands_tx, commands_rx) = mpsc::channel(32);

        let handle =
            { tokio::spawn(async move { client_main(requests_rx, commands_rx, stream).await }) };

        Ok(Client::new(requests_tx, commands_tx, handle))
    }

    /// Connect to an OVSDB server via UNIX domain socket.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use ovsdb::client::Client;
    ///
    /// let client = Client::connect_unix(Path::new("/var/run/openvswitch/db.sock"))
    ///     .await
    ///     .unwrap();
    /// ```
    pub async fn connect_unix(socket: &Path) -> Result<Self, ClientError> {
        let stream = UnixStream::connect(socket)
            .await
            .map_err(ClientError::ConnectionFailed)?;
        Client::start(stream).await
    }

    /// Disconnect from the OVSDB server and stop processing messages.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use ovsdb::client::Client;
    ///
    /// let client = Client::connect_unix(Path::new("/var/run/openvswitch/db.sock"))
    ///     .await
    ///     .unwrap();
    ///
    /// // Perform OVSDB operations
    ///
    /// client.stop().await.unwrap();
    pub async fn stop(mut self) -> Result<(), ClientError> {
        if let Some(sender) = self.command_sender.take() {
            sender
                .send(ClientCommand::Shutdown)
                .await
                .map_err(|e| ClientError::Internal(e.into()))?;
            drop(sender);
        };
        if let Some(sender) = self.request_sender.take() {
            drop(sender);
        }

        self.handle.await?
    }

    /// Execute a raw OVSDB request, receiving a raw response.
    ///
    /// Under normal circumstances, this method should only be called internally, with clients
    /// preferring one of the purpose-built methods (ie. [`Client::echo`]).  However, if for some reason
    /// those methods are insufficient, raw requests can be made to the database.
    ///
    /// ```rust,no_run
    ///
    /// use ovsdb::client::Client;
    /// use ovsdb::protocol::{request::Params, method::Method};
    ///
    /// struct MyParams {
    ///   values: Vec<i32>,
    /// }
    ///
    /// impl Params for MyParams {}
    ///
    /// let request = Request::new(Method::Echo, MyParams { values: vec![1, 2, 3] });
    ///
    /// let client = Client::connect_unix(Path::new("/var/run/openvswitch/db.sock"))
    ///     .await
    ///     .unwrap();
    ///
    /// if let Some(result) = client.execute(request).await.unwrap() {
    ///   println!("result: {}", result);
    /// }
    /// ```
    pub async fn execute<T>(&self, request: Request) -> Result<Option<T>, ClientError>
    where
        T: DeserializeOwned,
    {
        let (tx, rx) = oneshot::channel();

        match &self.request_sender {
            Some(s) => {
                s.send(ClientRequest { tx, request })
                    .await
                    .map_err(|e| ClientError::Internal(e.into()))?;
                let res = rx.await.map_err(|e| ClientError::Internal(e.into()))?;
                let r: Option<T> = res.result()?;
                Ok(r)
            }
            None => Err(ClientError::NotRunning),
        }
    }

    /// Issues an `echo` request to the OVSDB server.
    ///
    /// On success, the arguments to the request are returned as the result.
    ///
    /// ```rust,no_run
    ///
    /// use ovsdb::client::Client;
    ///
    /// let client = Client::connect_unix(Path::new("/var/run/openvswitch/db.sock"))
    ///     .await
    ///     .unwrap();
    ///
    /// let args = vec!["Hello", "OVSDB"];
    /// let result = client.echo(args.clone()).await.unwrap();
    /// assert_eq!(*result, args);
    /// ```
    pub async fn echo<T, I>(&self, args: T) -> Result<EchoResult, ClientError>
    where
        T: IntoIterator<Item = I> + Send,
        I: Into<String> + std::fmt::Debug,
    {
        match self
            .execute(crate::protocol::Request::new(
                Method::Echo,
                Some(Box::new(EchoParams::new(args))),
            ))
            .await?
        {
            Some(data) => Ok(data),
            None => Err(ClientError::UnexpectedResult),
        }
    }

    /// Issues a `list_dbs` request to the OVSDB server.
    ///
    /// On success, a list of databases supported by the server are returned.
    ///
    /// ```rust,no_run
    ///
    /// use ovsdb::client::Client;
    ///
    /// let client = Client::connect_unix(Path::new("/var/run/openvswitch/db.sock"))
    ///     .await
    ///     .unwrap();
    ///
    /// let dbs = client.list_databases().await.unwrap();
    /// println!("available databases: {}", dbs);
    /// ```
    pub async fn list_databases(&self) -> Result<ListDbsResult, ClientError> {
        match self
            .execute(crate::protocol::Request::new(Method::ListDatabases, None))
            .await?
        {
            Some(data) => Ok(data),
            None => Err(ClientError::UnexpectedResult),
        }
    }

    /// Issues a `get_schema` request to the OVSDB server.
    ///
    /// On success, a [Schema] instance is returned matching the OVSDB schema for the specified
    /// database.
    ///
    /// ```rust,no_run
    ///
    /// use ovsdb::client::Client;
    ///
    /// let client = Client::connect_unix(Path::new("/var/run/openvswitch/db.sock"))
    ///     .await
    ///     .unwrap();
    ///
    /// let schema = client.get_schema("Open_vSwitch").await.unwrap();
    /// println!("Open_vSwitch schema: {}", schema);
    /// ```
    pub async fn get_schema<S>(&self, database: S) -> Result<Schema, ClientError>
    where
        S: Into<String>,
    {
        match self
            .execute(crate::protocol::Request::new(
                Method::GetSchema,
                Some(Box::new(GetSchemaParams::new(database))),
            ))
            .await?
        {
            Some(data) => Ok(data),
            None => Err(ClientError::UnexpectedResult),
        }
    }

    /// Issues a `transact` request to the OVSDB server.
    ///
    /// TODO
    pub async fn transact<S, T>(
        &self,
        database: S,
        operations: Vec<Operation>,
    ) -> Result<T, ClientError>
    where
        S: Into<String>,
        T: DeserializeOwned,
    {
        match self
            .execute(crate::protocol::Request::new(
                Method::Transact,
                Some(Box::new(TransactParams::new(database, operations))),
            ))
            .await?
        {
            Some(data) => Ok(data),
            None => Err(ClientError::UnexpectedResult),
        }
    }
}

async fn client_main<T>(
    mut requests: mpsc::Receiver<ClientRequest>,
    mut commands: mpsc::Receiver<ClientCommand>,
    stream: T,
) -> Result<(), ClientError>
where
    T: AsyncReadExt + AsyncWriteExt,
{
    let (mut writer, mut reader) = Framed::new(stream, protocol::Codec::new()).split();
    let mut channels: HashMap<protocol::Uuid, oneshot::Sender<protocol::Response>> = HashMap::new();

    loop {
        tokio::select! {
            Some(req) = requests.recv() => {
                let request = req.request;
                if let Some(id) = request.id() {
                    channels.insert(*id, req.tx);
                }
                // writer.send(request.into()).await?;
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
                        if let Some(id) = res.id() {
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
