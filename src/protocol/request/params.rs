use std::fmt;

use serde::{Serialize, Serializer};

pub enum Params {
    Echo(Vec<String>),
    ListDatabases,
    GetSchema(String),
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
        }
    }
}
