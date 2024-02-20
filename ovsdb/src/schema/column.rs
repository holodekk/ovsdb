use serde::{
    de::{self, MapAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};
use serde_json::Value;

use super::Kind;

/// A single column of data in OVSDB.
#[derive(Clone, Debug, Default, Serialize)]
pub struct Column {
    name: String,
    kind: Kind,
    ephemeral: bool,
    mutable: bool,
}

impl Column {
    /// Name associated with this column in OVSDB.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn set_name<T>(&mut self, name: T)
    where
        T: Into<String>,
    {
        self.name = name.into();
    }

    /// Data type stored in this [Column].
    #[must_use]
    pub fn kind(&self) -> &Kind {
        &self.kind
    }

    /// Whether or not this value is ephemeral in nature.
    ///
    /// Ephemeral values are lost when the database is restarted.
    #[must_use]
    pub fn ephemeral(&self) -> bool {
        self.ephemeral
    }

    /// Whether or not the value in this column is mutable.
    ///
    /// Certain values cannot be modified after set initially with an insert.
    #[must_use]
    pub fn mutable(&self) -> bool {
        self.mutable
    }
}

impl<'de> Deserialize<'de> for Column {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ColumnVisitor;

        impl<'de> Visitor<'de> for ColumnVisitor {
            type Value = Column;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
                            column.ephemeral = e;
                        }
                        "mutable" => {
                            let m: bool = serde_json::from_value(v).map_err(de::Error::custom)?;
                            column.mutable = m;
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

    use crate::schema::{kind::RefType, Atomic};

    #[test]
    fn test_column_boolean() {
        let data = r#"{ "type": "boolean" }"#;
        let c: Column = serde_json::from_str(data).expect("Column");
        assert_eq!(c.kind.key().kind(), Atomic::Boolean);
        assert!(!c.ephemeral());
        assert!(!c.mutable());
    }

    #[test]
    fn test_column_integer() {
        let data = r#"{ "type": "integer" }"#;
        let c: Column = serde_json::from_str(data).expect("Column");
        assert!(matches!(c.kind.key().kind(), Atomic::Integer));
        assert!(!c.ephemeral());
        assert!(!c.mutable());
    }

    #[test]
    fn test_column_complex_integer() {
        let data = r#"{ "type": { "key": "integer" } }"#;
        let c: Column = serde_json::from_str(data).expect("Column");
        assert!(matches!(c.kind.key().kind(), Atomic::Integer));
        assert!(!c.ephemeral());
        assert!(!c.mutable());
    }

    #[test]
    fn test_column_complex_integer_with_constrints() {
        let data =
            r#"{ "type": { "key": { "type": "integer", "minInteger": 0, "maxInteger": 100 } } }"#;
        let c: Column = serde_json::from_str(data).expect("Column");
        assert!(matches!(c.kind.key().kind(), Atomic::Integer));
        assert_eq!(c.kind.key().min_integer(), Some(&0));
        assert_eq!(c.kind.key().max_integer(), Some(&100));
        assert!(!c.ephemeral());
        assert!(!c.mutable());
    }

    #[test]
    fn test_column_string_enum() {
        let data = r#"{ "type": { "key": { "type": "string", "enum": ["set", ["red", "blue", "green"]] } } }"#;
        let c: Column = serde_json::from_str(data).expect("Column");
        assert!(matches!(c.kind.key().kind(), Atomic::String));
        assert_eq!(
            c.kind.key().choices(),
            Some(&crate::protocol::Set(vec![
                "red".to_string(),
                "blue".to_string(),
                "green".to_string()
            ]))
        );
        assert!(!c.ephemeral());
        assert!(!c.mutable());
    }

    #[test]
    fn test_column_complex_uuid() {
        let data = r#"{ "type": { "key": { "type": "uuid", "refTable": "other_table", "refType": "weak" } }, "ephemeral": false, "mutable": true }"#;
        let c: Column = serde_json::from_str(data).expect("Column");
        assert!(matches!(c.kind.key().kind(), Atomic::Uuid));
        assert_eq!(c.kind.key().ref_table(), Some("other_table"));
        assert_eq!(c.kind.key().ref_type(), Some(RefType::Weak));
        assert!(!c.ephemeral());
        assert!(c.mutable());
    }

    #[test]
    fn test_column_simple_map() {
        let data = r#"{ "type": { "key": "string", "value": "string" } }"#;
        let c: Column = serde_json::from_str(data).expect("Column");
        assert!(matches!(c.kind.key().kind(), Atomic::String));
        assert!(matches!(
            c.kind.value().expect("value").kind(),
            Atomic::String
        ));
        assert!(!c.ephemeral());
        assert!(!c.mutable());
    }

    #[test]
    fn test_column_complex_map() {
        let data = r#"{ "type": { "key": { "type": "string", "enum": ["set", ["width", "height"]] }, "value": { "type": "string", "minLength": 5, "maxLength": 20 } } }"#;
        let c: Column = serde_json::from_str(data).expect("Column");
        assert!(matches!(c.kind.key().kind(), Atomic::String));
        assert_eq!(
            c.kind.key().choices(),
            Some(&crate::protocol::Set(vec![
                "width".to_string(),
                "height".to_string(),
            ]))
        );
        let value = c.kind.value().expect("value");
        assert!(matches!(&value.kind(), Atomic::String));
        assert!(matches!(&value.min_length(), Some(5)));
        assert!(matches!(&value.max_length(), Some(20)));
        assert!(!c.ephemeral());
        assert!(!c.mutable());
    }
}
