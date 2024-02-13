use serde::{
    de::{self, Deserializer, MapAccess, Visitor},
    Deserialize,
};
use serde_json::Value;

use super::Kind;

#[derive(Clone, Debug, Default)]
pub struct Column {
    pub name: String,
    pub kind: Kind,
    pub ephemeral: Option<bool>,
    pub mutable: Option<bool>,
}

impl<'de> Deserialize<'de> for Column {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ColumnVisitor;

        impl<'de> Visitor<'de> for ColumnVisitor {
            type Value = Column;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("`map`")
            }

            fn visit_map<S>(self, mut value: S) -> Result<Self::Value, S::Error>
            where
                S: MapAccess<'de>,
            {
                let mut column = Column::default();

                while let Some((k, v)) = value.next_entry::<String, Value>()? {
                    match k.as_str() {
                        "type" => {
                            let k: Kind = serde_json::from_value(v).map_err(de::Error::custom)?;
                            column.kind = k;
                        }
                        "ephemeral" => {
                            let e: bool = serde_json::from_value(v).map_err(de::Error::custom)?;
                            column.ephemeral = Some(e);
                        }
                        "mutable" => {
                            let m: bool = serde_json::from_value(v).map_err(de::Error::custom)?;
                            column.mutable = Some(m);
                        }
                        _ => Err(de::Error::unknown_field(
                            &k,
                            &["type", "ephemeral", "mutable"],
                        ))?,
                    }
                }
                Ok(column)
            }
        }

        deserializer.deserialize_map(ColumnVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::schema::{Atomic, RefType};

    #[test]
    fn test_column_boolean() {
        let data = r#"{ "type": "boolean" }"#;
        let c: Column = serde_json::from_str(data).unwrap();
        assert_eq!(c.kind.key.kind, Atomic::Boolean);
        assert_eq!(c.ephemeral, None);
        assert_eq!(c.mutable, None);
    }

    #[test]
    fn test_column_integer() {
        let data = r#"{ "type": "integer" }"#;
        let c: Column = serde_json::from_str(data).unwrap();
        assert!(matches!(c.kind.key.kind, Atomic::Integer));
        assert_eq!(c.ephemeral, None);
        assert_eq!(c.mutable, None);
    }

    #[test]
    fn test_column_complex_integer() {
        let data = r#"{ "type": { "key": "integer" } }"#;
        let c: Column = serde_json::from_str(data).unwrap();
        assert!(matches!(c.kind.key.kind, Atomic::Integer));
        assert_eq!(c.ephemeral, None);
        assert_eq!(c.mutable, None);
    }

    #[test]
    fn test_column_complex_integer_with_constrints() {
        let data =
            r#"{ "type": { "key": { "type": "integer", "minInteger": 0, "maxInteger": 100 } } }"#;
        let c: Column = serde_json::from_str(data).unwrap();
        assert!(matches!(c.kind.key.kind, Atomic::Integer));
        assert_eq!(c.kind.key.min_integer, Some(0));
        assert_eq!(c.kind.key.max_integer, Some(100));
        assert_eq!(c.ephemeral, None);
        assert_eq!(c.mutable, None);
    }

    #[test]
    fn test_column_string_enum() {
        let data = r#"{ "type": { "key": { "type": "string", "enum": ["set", ["red", "blue", "green"]] } } }"#;
        let c: Column = serde_json::from_str(data).unwrap();
        assert!(matches!(c.kind.key.kind, Atomic::String));
        assert_eq!(
            c.kind.key.choices,
            Some(crate::protocol::Set(vec![
                "red".to_string(),
                "blue".to_string(),
                "green".to_string()
            ]))
        );
        assert_eq!(c.ephemeral, None);
        assert_eq!(c.mutable, None);
    }

    #[test]
    fn test_column_complex_uuid() {
        let data = r#"{ "type": { "key": { "type": "uuid", "refTable": "other_table", "refType": "weak" } }, "ephemeral": false, "mutable": true }"#;
        let c: Column = serde_json::from_str(data).unwrap();
        assert!(matches!(c.kind.key.kind, Atomic::Uuid));
        assert_eq!(c.kind.key.ref_table.as_deref(), Some("other_table"));
        assert_eq!(c.kind.key.ref_type, Some(RefType::Weak));
        assert_eq!(c.ephemeral, Some(false));
        assert_eq!(c.mutable, Some(true));
    }

    #[test]
    fn test_column_simple_map() {
        let data = r#"{ "type": { "key": "string", "value": "string" } }"#;
        let c: Column = serde_json::from_str(data).unwrap();
        assert!(matches!(c.kind.key.kind, Atomic::String));
        assert!(matches!(c.kind.value.unwrap().kind, Atomic::String));
        assert_eq!(c.ephemeral, None);
        assert_eq!(c.mutable, None);
    }

    #[test]
    fn test_column_complex_map() {
        let data = r#"{ "type": { "key": { "type": "string", "enum": ["set", ["width", "height"]] }, "value": { "type": "string", "minLength": 5, "maxLength": 20 } } }"#;
        let c: Column = serde_json::from_str(data).unwrap();
        assert!(matches!(c.kind.key.kind, Atomic::String));
        assert_eq!(
            c.kind.key.choices,
            Some(crate::protocol::Set(vec![
                "width".to_string(),
                "height".to_string(),
            ]))
        );
        let value = c.kind.value.unwrap();
        assert!(matches!(&value.kind, Atomic::String));
        assert!(matches!(&value.min_length, Some(5)));
        assert!(matches!(&value.max_length, Some(20)));
        assert_eq!(c.ephemeral, None);
        assert_eq!(c.mutable, None);
    }
}
