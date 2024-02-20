use std::ops::Deref;

use serde::{Deserialize, Serialize};

/// The result returned by the `list_dbs` method.
///
/// Will be a list of databases supported by the connected OVSDB server.
#[derive(Debug, Deserialize, Serialize)]
pub struct ListDbsResult(Vec<String>);

impl Deref for ListDbsResult {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
