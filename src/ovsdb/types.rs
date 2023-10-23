use serde::ser::SerializeSeq;
use serde::{Deserialize, Serialize, Serializer};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
#[allow(non_camel_case_types)]
pub enum OVSAtom {
    set(String, Vec<OVSAtom>),
    uuid(String, Uuid),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OVSBridge {
    pub _uuid: OVSAtom,
    pub name: String,
    pub datapath_type: String,
    pub ports: OVSAtom,
}

#[derive(Deserialize, Serialize)]
pub struct OVSPort {
    _uuid: OVSAtom,
    name: String,
    interfaces: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct OVSDBListResult<T> {
    pub rows: Vec<T>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "op")]
pub enum OVSDBOperation {
    #[serde(rename = "select")]
    Select {
        table: String,
        #[serde(rename = "where")]
        clauses: Vec<String>,
    },
}

#[derive(Debug)]
pub struct OVSDBTransactParams {
    pub database: String,
    pub operations: Vec<OVSDBOperation>,
}

impl Serialize for OVSDBTransactParams {
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
pub enum OVSDBMethod {
    #[serde(rename = "transact")]
    Transact(OVSDBTransactParams),
}

#[derive(Debug, Serialize)]
pub struct OVSDBRequest {
    #[serde(flatten)]
    pub method: OVSDBMethod,
    pub id: String,
}
