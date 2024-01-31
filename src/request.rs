use serde::{
    ser::{SerializeMap, SerializeSeq},
    Serialize, Serializer,
};

pub enum Method {
    Echo,
    ListDatabases,
    GetSchema(GetSchemaParams),
    // Transact(TransactParams),
    // Cancel(String),
    // Monitor(MonitorParams),
    // Update(UpdateParams),
    // MonitorCancel(String),
    // Lock(String),
    // Steal(String),
    // Unlock(String),
    // Locked(String),
    // Stolen(String),
}

impl Method {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Echo => "echo",
            Self::ListDatabases => "list_dbs",
            Self::GetSchema(_) => "get_schema",
        }
    }

    pub fn params(&self) -> Option<impl Serialize + '_> {
        match self {
            Self::Echo => None,
            Self::ListDatabases => None,
            Self::GetSchema(params) => Some(params),
        }
    }
}

pub struct GetSchemaParams {
    name: String,
}

impl GetSchemaParams {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

impl Serialize for GetSchemaParams {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(1))?;
        seq.serialize_element(&self.name)?;
        seq.end()
    }
}

pub struct Request<'a> {
    pub id: String,
    pub method: &'a Method,
}

impl<'a> Serialize for Request<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(3))?;
        map.serialize_entry("id", &self.id)?;
        map.serialize_entry("method", self.method.name())?;
        if let Some(p) = self.method.params() {
            map.serialize_entry("params", &p)?;
        } else {
            let p: Vec<String> = vec![];
            map.serialize_entry("params", &p)?;
        }
        map.end()
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "op")]
pub enum Operation {
    #[serde(rename = "select")]
    Select {
        table: String,
        #[serde(rename = "where")]
        clauses: Vec<String>,
    },
}

#[derive(Debug)]
pub struct TransactParams {
    pub database: String,
    pub operations: Vec<Operation>,
}

impl Serialize for TransactParams {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.operations.len() + 1))?;
        seq.serialize_element(&self.database)?;
        for op in &self.operations {
            seq.serialize_element(&op)?;
        }
        seq.end()
    }
}
