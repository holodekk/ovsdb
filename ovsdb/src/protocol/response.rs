use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

use crate::{Error::ParseError, Result};

#[derive(Debug, Deserialize, Serialize)]
pub struct Response {
    pub id: Option<super::Uuid>,
    result: Option<Value>,
    pub error: Option<String>,
}

impl Response {
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
}

#[derive(Debug, Deserialize)]
pub struct ListResult<T> {
    pub rows: Vec<T>,
}
