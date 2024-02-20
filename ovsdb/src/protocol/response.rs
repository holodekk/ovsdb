use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

use crate::{Error::ParseError, Result};

/// A response to an OVSDB method call.
#[derive(Debug, Deserialize, Serialize)]
pub struct Response {
    id: Option<super::Uuid>,
    result: Option<Value>,
    error: Option<String>,
}

impl Response {
    /// Id of the original request (used for synchronization)
    #[must_use]
    pub fn id(&self) -> Option<&super::Uuid> {
        self.id.as_ref()
    }

    /// Data returned by the server in response to a method call.
    pub fn result<T>(&self) -> Result<Option<T>>
    where
        T: DeserializeOwned,
    {
        match &self.result {
            Some(r) => {
                let v: T = serde_json::from_value(r.clone()).map_err(ParseError)?;
                Ok(Some(v))
            }
            None => Ok(None),
        }
    }

    /// Any error encountered by the server in processing the method call.
    #[must_use]
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }
}

/// Response to a `query` transact method call.
#[derive(Debug, Deserialize)]
pub struct ListResult<T> {
    rows: Vec<T>,
}

impl<T> ListResult<T> {
    /// Rows returned by the server.
    #[must_use]
    pub fn rows(&self) -> &Vec<T> {
        &self.rows
    }
}
