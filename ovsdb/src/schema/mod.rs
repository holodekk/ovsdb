use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use serde::{Deserialize, Deserializer};
use serde_json::Value;

mod column;
pub use column::Column;
mod kind;
pub use kind::{Constraints, Kind, RefType};
mod table;
pub use table::Table;

#[derive(thiserror::Error, Debug)]
pub enum Error {}

#[derive(Debug, Deserialize)]
pub struct Schema {
    pub name: String,
    pub version: String,
    pub cksum: String,
    #[serde(deserialize_with = "deserialize_tables")]
    pub tables: Vec<Table>,
}

fn deserialize_tables<'de, D>(de: D) -> Result<Vec<Table>, D::Error>
where
    D: Deserializer<'de>,
{
    Value::deserialize(de)?
        .as_object()
        .expect("convert schema `tables` to json object")
        .iter()
        .map(|(k, v)| -> Result<Table, serde_json::Error> {
            let mut t: Table = Table::deserialize(v)?;
            t.name = k.to_string();
            Ok(t)
        })
        .collect::<Result<Vec<Table>, serde_json::Error>>()
        .map_err(serde::de::Error::custom)
}

impl Schema {
    pub fn from_file<P>(filename: P) -> Result<Self, std::io::Error>
    where
        P: AsRef<Path>,
    {
        let mut schema_file = File::open(filename)?;
        let mut schema_contents = String::new();
        schema_file.read_to_string(&mut schema_contents)?;

        let schema: Schema = serde_json::from_slice(schema_contents.as_bytes())?;

        Ok(schema)
    }
}

impl std::str::FromStr for Schema {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}
