use serde::de::DeserializeOwned;
use serde::ser::SerializeSeq;
use serde::{Deserialize, Serialize, Serializer};
use std::cell::Ref;
use uuid::Uuid;

mod connection;
pub use connection::*;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unknown Error")]
    Unknown,
    #[error("Not Connected")]
    NotConnected,
    #[error("Unexpected IO Error")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
#[allow(non_camel_case_types)]
pub enum Atom {
    set(String, Vec<Atom>),
    uuid(String, Uuid),
}

#[derive(Debug, Deserialize)]
pub struct ListResult<T> {
    pub rows: Vec<T>,
}

#[derive(Debug, Deserialize)]
pub struct ListResponse<T> {
    pub result: Vec<ListResult<T>>,
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

#[derive(Debug, Serialize)]
#[serde(tag = "method", content = "params")]
pub enum Method {
    #[serde(rename = "transact")]
    Transact(TransactParams),
}

#[derive(Debug, Serialize)]
pub struct Request {
    #[serde(flatten)]
    pub method: Method,
    pub id: String,
}

#[derive(Default)]
pub struct RequestBuilder {
    id: String,
    database: String,
    operations: Vec<Operation>,
}

pub trait Entity {
    fn table() -> &'static str;
}

impl RequestBuilder {
    pub fn new(database: &str) -> RequestBuilder {
        Self {
            id: String::from("123456"),
            database: database.to_string(),
            operations: vec![],
        }
    }

    pub fn query(mut self, table: &str) -> RequestBuilder {
        self.operations.push(Operation::Select {
            table: table.to_string(),
            clauses: vec![],
        });
        self
    }

    pub fn build(self) -> Request {
        Request {
            id: self.id,
            method: Method::Transact(TransactParams {
                database: self.database,
                operations: self.operations,
            }),
        }
    }
}

pub trait Client<T>
where
    T: Connection,
{
    fn disconnect(&self) -> Result<(), Error>;
    fn conn(&self) -> Ref<T>;
    fn database(&self) -> &str;

    fn list<R>(&self) -> Result<Option<Vec<R>>, Error>
    where
        R: Entity + DeserializeOwned,
    {
        let request = RequestBuilder::new(self.database())
            .query(R::table())
            .build();

        let conn = self.conn();
        conn.send(request).unwrap();
        let res: ListResponse<R> = serde_json::from_slice(&conn.recv().unwrap()).unwrap();
        match res.result.into_iter().next() {
            Some(result) => Ok(Some(result.rows)),
            None => Ok(None),
        }
    }
}
