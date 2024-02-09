pub mod protocol;
pub mod schema;

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
    // #[error("Internal synchronization error")]
    // Synchronization(#[from] mpsc::error::SendError<ClientRequest>),
    // #[error("Internal synchronization error")]
    // InternalSync(#[from] mpsc::error::SendError<ClientCommand>),
    #[error("Shutdown error")]
    Shutdown(#[from] tokio::task::JoinError),
    #[error("Tokio receive")]
    TokioReceive(#[from] tokio::sync::oneshot::error::RecvError),
    // #[error("Protocol error")]
    // Codec(#[from] protocol::codec::Error),
}
