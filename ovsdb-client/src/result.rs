use ovsdb::protocol::CodecError;
use tokio::sync::{mpsc::error::SendError, oneshot::error::RecvError};

use super::client::{ClientCommand, ClientRequest};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("An error occurred when communicating with the server")]
    CommunicationFailure(#[from] CodecError),
    #[error("Unexpected result received in response object")]
    UnexpectedResult,
    #[error("Failed to deliver command")]
    InternalCommandFailure(#[from] SendError<ClientCommand>),
    #[error("Failed to deliver request")]
    InternalRequestFailure(#[from] SendError<ClientRequest>),
    #[error("Failed to receive response")]
    InternalResponseError(#[from] RecvError),
    #[error("OVSDB error")]
    OvsdbError(#[from] ovsdb::Error),
    #[error("Failed to establish connection with the server")]
    ConnectionFailed(#[source] std::io::Error),
    #[error("Failed to shutdown client")]
    ShutdownError(#[from] tokio::task::JoinError),
    #[error("Client thread not active")]
    NotRunning,
}

pub type Result<T> = std::result::Result<T, Error>;
