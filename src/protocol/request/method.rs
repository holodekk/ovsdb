use serde::{Serialize, Serializer};

#[derive(Debug, PartialEq)]
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

impl Serialize for Method {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let method = match self {
            Self::Echo => "echo",
            Self::ListDatabases => "list_dbs",
            Self::GetSchema => "get_schema",
        };
        method.serialize(serializer)
    }
}
