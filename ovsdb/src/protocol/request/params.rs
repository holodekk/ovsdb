use std::fmt;

use serde::{ser::SerializeSeq, Deserialize, Serialize, Serializer};

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "op")]
pub enum Operation {
    #[serde(rename = "select")]
    Select {
        table: String,
        #[serde(rename = "where")]
        clauses: Vec<String>,
    },
}

#[derive(Debug, Deserialize)]
pub struct TransactParams {
    database: String,
    operations: Vec<Operation>,
}

impl TransactParams {
    pub fn new<S>(database: S, operations: Vec<Operation>) -> Self
    where
        S: Into<String>,
    {
        Self {
            database: database.into(),
            operations,
        }
    }
}

#[derive(Debug)]
pub enum Params {
    Echo(Vec<String>),
    ListDatabases,
    GetSchema(String),
    Transact(TransactParams),
}

impl Serialize for Params {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Echo(p) => p.serialize(serializer),
            Self::ListDatabases => Vec::<String>::new().serialize(serializer),
            Self::GetSchema(s) => vec![s].serialize(serializer),
            Self::Transact(TransactParams {
                database,
                operations,
            }) => {
                let mut seq = serializer.serialize_seq(Some(operations.len() + 1))?;
                seq.serialize_element(database)?;
                for op in operations {
                    seq.serialize_element(op)?;
                }
                seq.end()
            }
        }
    }
}

impl fmt::Display for Params {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Echo(p) => {
                write!(f, "[")?;
                for (idx, item) in p.iter().enumerate() {
                    if idx > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            Self::ListDatabases => write!(f, "[]"),
            Self::GetSchema(p) => {
                write!(f, "{}", p)
            }
            Self::Transact(TransactParams {
                database,
                operations,
            }) => {
                write!(f, "database => {}, ", database)?;
                write!(f, "operations => [")?;
                for (idx, op) in operations.iter().enumerate() {
                    if idx > 0 {
                        write!(f, ", ")?;
                    }
                    match op {
                        Operation::Select { table, .. } => {
                            write!(f, "{{select => {}}}", table)?;
                        }
                    }
                }
                write!(f, "]")
            }
        }
    }
}
