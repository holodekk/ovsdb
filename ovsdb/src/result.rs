#[cfg(feature = "protocol")]
use crate::protocol::CodecError;

/// This type represents all errors that can occur within OVSDB.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// A failure occurrecd while parsing JSON data.
    #[error("Failed to parse JSON data")]
    ParseError(#[source] serde_json::Error),
    /// A provided file path does not exist on disk.
    #[error("File not found")]
    FileNotFound(#[source] std::io::Error),
    /// Unable to open a file due to permissions issues.
    #[error("Permission denied")]
    PermissionDenied(#[source] std::io::Error),
    /// A general IO error occurred while reading data from a file.
    #[error("Error reading data from file")]
    ReadError(#[source] std::io::Error),
    #[cfg(feature = "protocol")]
    /// A failure occurred while processing communications between client and server.
    #[error("An error occurred when communicating with the server")]
    CommunicationFailure(#[from] CodecError),
}

/// Alias for a [Result][std::result::Result] with the error type [Error].
pub type Result<T> = std::result::Result<T, Error>;
