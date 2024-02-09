use serde::{de::DeserializeOwned, Serialize};

mod codec;
pub use codec::Codec;

mod request;
pub use request::*;
mod response;
pub use response::*;

pub mod enumeration;
mod map;
pub use map::*;
mod message;
pub use message::Message;
mod set;
pub use set::*;
mod uuid;
pub use self::uuid::*;

#[derive(thiserror::Error, Debug)]
pub enum Error {}

pub fn encode<T>(value: T) -> Result<Vec<u8>, crate::Error>
where
    T: Serialize,
{
    let res = serde_json::to_vec(&value)?;
    Ok(res)
}

pub fn decode<T>(value: serde_json::Value) -> Result<T, crate::Error>
where
    T: DeserializeOwned,
{
    let res = serde_json::from_value::<T>(value)?;
    Ok(res)
}
