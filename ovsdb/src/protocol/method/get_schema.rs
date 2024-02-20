use std::ops::Deref;

use serde::{Deserialize, Serialize};

use crate::schema::Schema;

use super::Params;

/// Parameters for the `get_schema` OVSDB method.
///
/// This is merely a NewType around a String, indicating which databases's schema should be
/// retrieved..
#[derive(Debug, Deserialize, Serialize)]
pub struct GetSchemaParams(String);

impl GetSchemaParams {
    /// Create a new set of `get_schema` params.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ovsdb::protocol::method::GetSchemaParams;
    ///
    /// let params = GetSchemaParams::new("Open_vSwitch");
    /// ```
    pub fn new<T>(database: T) -> Self
    where
        T: Into<String>,
    {
        Self(database.into())
    }
}

impl Params for GetSchemaParams {}

/// The result returned by the `get_schema` method.
#[derive(Debug, Deserialize, Serialize)]
pub struct GetSchemaResult(Schema);

impl Deref for GetSchemaResult {
    type Target = Schema;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
