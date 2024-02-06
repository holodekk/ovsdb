use serde::{ser::SerializeMap, Serialize, Serializer};

use crate::protocol::Value;

#[derive(Debug)]
pub struct EchoParams(pub Vec<Value>);

#[derive(Debug)]
pub enum Method {
    Echo,
    ListDatabases,
    GetSchema,
    // Transact,
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

impl Method {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Echo => "echo",
            Self::ListDatabases => "list_dbs",
            Self::GetSchema => "get_schema",
        }
    }
}

pub struct Request {
    pub id: uuid::Uuid,
    pub method: Method,
    pub params: Option<Vec<Value>>,
}

impl Request {
    pub fn new(method: Method, params: Option<Vec<Value>>) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            method,
            params,
        }
    }
}

impl Serialize for Request {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(3))?;
        map.serialize_entry("id", &self.id)?;
        map.serialize_entry("method", self.method.name())?;
        map.serialize_entry("params", &self.params)?;
        map.end()
    }
}
