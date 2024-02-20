use serde::{de, Deserialize, Deserializer, Serialize};
use serde_json::Value;

use super::column::Column;

/// An OVSDB table containing rows of structured data.
#[derive(Debug, Deserialize, Serialize)]
pub struct Table {
    #[serde(default)]
    name: String,
    #[serde(rename = "isRoot", default)]
    is_root: bool,
    #[serde(rename = "maxRows", default)]
    max_rows: Option<i64>,
    #[serde(deserialize_with = "deserialize_columns")]
    columns: Vec<Column>,
}

impl Table {
    pub(crate) fn set_name<T>(&mut self, name: T)
    where
        T: Into<String>,
    {
        self.name = name.into();
    }

    /// Name of the OVSDB table.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Whether table is a root table.
    ///
    /// Records in root tables are allowed to exist without being referenced.
    #[must_use]
    pub fn is_root(&self) -> bool {
        self.is_root
    }

    /// If present, the maximum number of records that may exist in the table.
    #[must_use]
    pub fn max_rows(&self) -> Option<i64> {
        self.max_rows
    }

    /// List of columns present in the table.
    #[must_use]
    pub fn columns(&self) -> &Vec<Column> {
        &self.columns
    }
}

fn deserialize_columns<'de, D>(de: D) -> Result<Vec<Column>, D::Error>
where
    D: Deserializer<'de>,
{
    Value::deserialize(de)?
        .as_object()
        .expect("convert table `columns` to json object")
        .iter()
        .map(|(k, v)| {
            let mut c: Column = Column::deserialize(v).map_err(de::Error::custom)?;
            c.set_name(k);
            Ok(c)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_table() {
        let data = r#"{ "columns": { "name": { "type": "string", "mutable": false } }, "isRoot": false, "maxRows": 100 }"#;
        let t: Table = serde_json::from_str(data).expect("Table");
        assert_eq!(t.columns.len(), 1);
        assert!(!t.is_root());
        assert_eq!(t.max_rows(), Some(100));
    }
}
