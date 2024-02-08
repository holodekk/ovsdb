use serde::{Deserialize, Deserializer};
use serde_json::Value;

use super::column::Column;

#[derive(Debug, Deserialize)]
pub struct Table {
    #[serde(default)]
    pub name: String,
    #[serde(rename = "isRoot", default)]
    pub is_root: Option<bool>,
    #[serde(rename = "maxRows", default)]
    pub max_rows: Option<i64>,
    #[serde(deserialize_with = "deserialize_columns")]
    pub columns: Vec<Column>,
}

fn deserialize_columns<'de, D>(de: D) -> Result<Vec<Column>, D::Error>
where
    D: Deserializer<'de>,
{
    let columns = Value::deserialize(de)?
        .as_object()
        .expect("convert table `columns` to json object")
        .iter()
        .map(|(k, v)| {
            let mut c: Column = Column::deserialize(v).unwrap();
            c.name = k.to_string();
            c
        })
        .collect();
    Ok(columns)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_table() {
        let data = r#"{ "columns": { "name": { "type": "string", "mutable": false } }, "isRoot": false, "maxRows": 100 }"#;
        let t: Table = serde_json::from_str(data).unwrap();
        assert_eq!(t.columns.len(), 1);
        assert_eq!(t.is_root, Some(false));
        assert_eq!(t.max_rows, Some(100));
    }
}
