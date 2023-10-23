mod types;
pub use types::*;

#[derive(thiserror::Error, Debug)]
pub enum OVSDBError {
    #[error("Unknown Error")]
    Unknown,
    #[error("Not Connected")]
    NotConnected,
    #[error("Unexpected IO Error")]
    Io(#[from] std::io::Error),
}
