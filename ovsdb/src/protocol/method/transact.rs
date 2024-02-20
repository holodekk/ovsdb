use serde::{ser::SerializeSeq, Deserialize, Serialize, Serializer};

use super::Params;

/// OVSDB operation to be performed.  Somewhat analgous to a SQL statement.
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "op")]
pub enum Operation {
    /// An OVSDB `select` operation
    #[serde(rename = "select")]
    Select {
        /// The [Table][crate::schema::Table] to operate against.
        table: String,
        /// A collection of clauses to act as filters against the table data.
        #[serde(rename = "where")]
        clauses: Vec<String>,
    },
}

/// Parameters for the `transact` OVSDB method.
#[derive(Debug, Deserialize)]
pub struct TransactParams {
    database: String,
    operations: Vec<Operation>,
}

impl TransactParams {
    /// Create a new set of `transact` parameters.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ovsdb::protocol::method::{Operation, TransactParams};
    ///
    /// let op = Operation::Select { table: "Bridges".into(), clauses: vec![] };
    /// let params = TransactParams::new("Bridges", vec![op]);
    /// ```
    pub fn new<T>(database: T, operations: Vec<Operation>) -> Self
    where
        T: Into<String>,
    {
        Self {
            database: database.into(),
            operations,
        }
    }
}

impl Params for TransactParams {}

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
