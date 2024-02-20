//! Available OVSDB methods.

use erased_serde::Serialize as ErasedSerialize;
use serde::{Serialize, Serializer};

mod echo;
pub use echo::{EchoParams, EchoResult};

mod get_schema;
pub use get_schema::{GetSchemaParams, GetSchemaResult};

mod list_dbs;
pub use list_dbs::ListDbsResult;

mod transact;
pub use transact::{Operation, TransactParams};

/// OVSDB method.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Method {
    /// OVSDB `echo` method.
    Echo,
    /// OVSDB `list_dbs` method.
    ListDatabases,
    /// OVSDB `get_schema` method.
    GetSchema,
    /// OVSDB `transact` method.
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

/// Trait specifying requirements for a valid OVSDB wire request.
///
/// Primary exists to ensure type-safety.
pub trait Params: ErasedSerialize + Send + std::fmt::Debug {}
erased_serde::serialize_trait_object!(Params);
