//! OVSDB Schema processing.
use std::fs;
use std::path::Path;

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

mod atomic;
pub use atomic::Atomic;
mod column;
pub use column::Column;
mod kind;
pub use kind::{BaseKind, Kind, RefType};
mod table;
pub use table::Table;

use crate::{Error, Result};

/// Specification for the tables making up an OVSDB database.
#[derive(Debug, Deserialize, Serialize)]
pub struct Schema {
    name: String,
    version: String,
    cksum: String,
    #[serde(deserialize_with = "deserialize_tables")]
    tables: Vec<Table>,
}

impl Schema {
    /// Name of this database schema.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Version of the scheme.
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Internal checksum for the schema.
    ///
    /// Used by schema developers.  NOT intended for clients.
    #[must_use]
    pub fn cksum(&self) -> &str {
        &self.cksum
    }

    /// List of table schemas supported by this database schema.
    #[must_use]
    pub fn tables(&self) -> &Vec<Table> {
        &self.tables
    }
}

fn deserialize_tables<'de, D>(de: D) -> std::result::Result<Vec<Table>, D::Error>
where
    D: Deserializer<'de>,
{
    Value::deserialize(de)?
        .as_object()
        .expect("convert schema `tables` to json object")
        .iter()
        .map(|(k, v)| -> std::result::Result<Table, serde_json::Error> {
            let mut t: Table = Table::deserialize(v)?;
            t.set_name(k);
            Ok(t)
        })
        .collect::<std::result::Result<Vec<Table>, serde_json::Error>>()
        .map_err(serde::de::Error::custom)
}

impl Schema {
    /// Load an OVSDB [Schema] from a file on disk.
    pub fn from_file<P>(filename: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let schema_contents = fs::read(filename).map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => Error::FileNotFound(e),
            std::io::ErrorKind::PermissionDenied => Error::PermissionDenied(e),
            _ => Error::ReadError(e),
        })?;

        let schema: Schema = serde_json::from_slice(&schema_contents).map_err(Error::ParseError)?;

        Ok(schema)
    }
}

impl std::str::FromStr for Schema {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let schema: Schema = serde_json::from_str(s).map_err(Error::ParseError)?;
        Ok(schema)
    }
}
