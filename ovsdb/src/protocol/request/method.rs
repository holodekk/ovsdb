use std::convert::TryFrom;

use serde::{Serialize, Serializer};

#[derive(Debug, PartialEq)]
pub enum Method {
    Echo,
    ListDatabases,
    GetSchema,
    Transact,
    // Cancel,
    // Monitor,
    // Update,
    // MonitorCancel,
    // Lock,
    // Steal,
    // Unlock,
    // Locked,
    // Stolen,
}

impl Serialize for Method {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let method = match self {
            Self::Echo => "echo",
            Self::ListDatabases => "list_dbs",
            Self::GetSchema => "get_schema",
            Self::Transact => "transact",
        };
        method.serialize(serializer)
    }
}

impl TryFrom<String> for Method {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "echo" => Ok(Self::Echo),
            "list_dbs" => Ok(Self::ListDatabases),
            "get_schema" => Ok(Self::GetSchema),
            "transact" => Ok(Self::Transact),
            _ => Err(format!("Invalid method: {}", value)),
        }
    }
}
