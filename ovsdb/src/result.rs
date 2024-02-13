#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to parse JSON data")]
    ParseError(#[source] serde_json::Error),
    #[error("File not found")]
    FileNotFound(#[source] std::io::Error),
    #[error("Permission denied")]
    PermissionDenied(#[source] std::io::Error),
    #[error("Error reading data from file")]
    ReadError(#[source] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
