use std::ops::Deref;

use serde::{Deserialize, Serialize};

use super::Params;

/// Parameters for the OVSDB `echo` method.
///
/// These parameters are merely returned untouched by the server.
#[derive(Debug, Deserialize, Serialize)]
pub struct EchoParams(Vec<String>);

impl EchoParams {
    /// Create a new set of echo parameters.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ovsdb::protocol::method::EchoParams;
    ///
    /// let params = EchoParams::new(vec!["Hello", "OVSDB"]);
    /// ```
    pub fn new<T, I>(args: T) -> Self
    where
        T: IntoIterator<Item = I>,
        I: Into<String>,
    {
        Self(args.into_iter().map(|s| s.into()).collect())
    }
}

impl Params for EchoParams {}

/// Result returned by OVSDB for the `echo` method.
#[derive(Debug, Deserialize, Serialize)]
pub struct EchoResult(Vec<String>);

impl Deref for EchoResult {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
