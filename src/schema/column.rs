use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use serde::{Deserialize, Deserializer};
use serde_json::Value;

use super::DataType;

#[derive(Clone, Debug)]
pub struct Column {
    pub name: String,
    pub kind: DataType,
    pub min: Option<i64>,
    pub max: Option<i64>,
    pub ephemeral: Option<bool>,
    pub mutable: Option<bool>,
}

impl Column {
    pub fn is_set(&self) -> bool {
        match self.kind {
            DataType::Map { .. } => false,
            _ => {
                if self.min.is_some()
                    && self.max.is_some()
                    && (self.min.unwrap() != 1 || self.max.unwrap() != 1)
                {
                    if self.kind.is_enum() {
                        if self.max.unwrap() != 1 {
                            return true;
                        }
                    } else {
                        return true;
                    }
                }

                false
            }
        }
    }

    pub fn is_optional(&self) -> bool {
        self.min.is_some() && self.max.is_some() && self.min.unwrap() == 0 && self.max.unwrap() == 1
    }
}

impl<'de> Deserialize<'de> for Column {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut min = None;
        let mut max = None;

        let data = Value::deserialize(de)?;

        let obj = data.as_object().unwrap();

        let ephemeral = obj
            .get("ephemeral")
            .map(|e| e.as_bool().expect("convert `ephemeral` to `bool`"));
        let mutable = obj
            .get("mutable")
            .map(|m| m.as_bool().expect("convert `mutable` to `bool`"));

        let kind = DataType::from_value(&data).unwrap();

        if obj.get("type").unwrap().is_object() {
            let typ = obj.get("type").unwrap().as_object().unwrap();

            min = typ.get("min").map(|m| m.as_i64().unwrap());
            max = typ.get("max").map(|m| match m {
                Value::String(v) => match v.as_str() {
                    "unlimited" => -1,
                    _ => panic!("Unexpected string value for max: {}", v),
                },
                Value::Number(v) => v.as_i64().unwrap(),
                _ => panic!("Unexpected type for max: {}", m),
            });
        }

        Ok(Column {
            name: "".to_string(),
            kind,
            min,
            max,
            ephemeral,
            mutable,
        })
    }
}

impl ToTokens for Column {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let attr_name = match &self.name == "type" {
            true => format_ident!("{}", "kind"),
            _ => format_ident!("{}", &self.name.to_case(Case::Snake)),
        };
        let attr_type = match self.is_set() {
            true => {
                let kind = self.kind.to_token_stream();
                quote! { Vec<#kind> }
            }
            false => self.kind.to_token_stream(),
        };
        let mut serializer: String;
        let mut deserializer: String;
        if self.is_set() {
            serializer = "from_set".to_string();
            deserializer = "to_set".to_string();
        } else {
            match self.kind {
                DataType::Unknown => {
                    unimplemented!()
                }
                DataType::Boolean => {
                    serializer = "from_bool".to_string();
                    deserializer = "to_bool".to_string();
                }
                DataType::Integer(_) => {
                    serializer = "from_i64".to_string();
                    deserializer = "to_i64".to_string();
                }
                DataType::Real(_) => {
                    serializer = "from_f64".to_string();
                    deserializer = "to_f64".to_string();
                }
                DataType::String(_) => {
                    serializer = "from_string".to_string();
                    deserializer = "to_string".to_string();
                }
                DataType::Map { .. } => {
                    serializer = "from_map".to_string();
                    deserializer = "to_map".to_string();
                }
                DataType::Uuid { .. } => {
                    serializer = "from_uuid".to_string();
                    deserializer = "to_uuid".to_string();
                }
            }
        }
        serializer = format!("protocol::{}", serializer);
        deserializer = format!("protocol::{}", deserializer);

        // #[serde(flatten)]
        tokens.extend(quote! {
            #[serde(serialize_with = #serializer, deserialize_with = #deserializer)]
            pub #attr_name: #attr_type
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::schema::RefType;

    #[test]
    fn handles_boolean() {
        let data = r#"{ "type": "boolean" }"#;
        let c: Column = serde_json::from_str(data).unwrap();
        assert_eq!(c.kind, DataType::Boolean);
    }

    #[test]
    fn handles_scalar_integer() {
        let data = r#"{ "type": "integer" }"#;
        let c: Column = serde_json::from_str(data).unwrap();
        assert!(matches!(c.kind, DataType::Integer(_)));
    }

    #[test]
    fn handles_complex_integer() {
        let data = r#"{ "type": { "key": "integer" } }"#;
        let c: Column = serde_json::from_str(data).unwrap();
        assert!(matches!(c.kind, DataType::Integer(_)));
    }

    #[test]
    fn handles_complex_integer_with_constrints() {
        let data =
            r#"{ "type": { "key": { "type": "integer", "minInteger": 0, "maxInteger": 100 } } }"#;
        let c: Column = serde_json::from_str(data).unwrap();
        assert!(matches!(c.kind, DataType::Integer(_)));
        if let DataType::Integer(constraints) = c.kind {
            assert_eq!(constraints.min, Some(0));
            assert_eq!(constraints.max, Some(100));
            assert_eq!(constraints.options, None);
        } else {
            panic!()
        }
    }

    #[test]
    fn handles_integer_enum() {
        let data = r#"{ "type": { "key": { "type": "integer", "enum": ["set", [0, 1, 2]] } } }"#;
        let c: Column = serde_json::from_str(data).unwrap();
        assert!(matches!(c.kind, DataType::Integer(_)));
        if let DataType::Integer(constraints) = c.kind {
            assert_eq!(constraints.min, None);
            assert_eq!(constraints.max, None);
            assert_eq!(constraints.options, Some(vec![0, 1, 2]));
        } else {
            panic!()
        }
    }

    #[test]
    fn handles_scalar_real() {
        let data = r#"{ "type": "real" }"#;
        let c: Column = serde_json::from_str(data).unwrap();
        assert!(matches!(c.kind, DataType::Real(_)));
    }

    #[test]
    fn handles_complex_real() {
        let data = r#"{ "type": { "key": "real" } }"#;
        let c: Column = serde_json::from_str(data).unwrap();
        assert!(matches!(c.kind, DataType::Real(_)));
    }

    #[test]
    fn handles_complex_real_with_constrints() {
        let data = r#"{ "type": { "key": { "type": "real", "minReal": 1.1, "maxReal": 2.2 } } }"#;
        let c: Column = serde_json::from_str(data).unwrap();
        if let DataType::Real(constraints) = c.kind {
            assert_eq!(constraints.min, Some(1.1));
            assert_eq!(constraints.max, Some(2.2));
            assert_eq!(constraints.options, None);
        } else {
            panic!()
        }
    }

    #[test]
    fn handles_real_enum() {
        let data = r#"{ "type": { "key": { "type": "real", "enum": ["set", [1.1, 2.2, 3.3]] } } }"#;
        let c: Column = serde_json::from_str(data).unwrap();
        if let DataType::Real(constraints) = c.kind {
            assert_eq!(constraints.min, None);
            assert_eq!(constraints.max, None);
            assert_eq!(constraints.options, Some(vec![1.1, 2.2, 3.3]));
        } else {
            panic!()
        }
    }

    #[test]
    fn handles_scalar_string() {
        let data = r#"{ "type": "string" }"#;
        let c: Column = serde_json::from_str(data).unwrap();
        assert!(matches!(c.kind, DataType::String(_)));
    }

    #[test]
    fn handles_complex_string() {
        let data = r#"{ "type": { "key": "string" } }"#;
        let c: Column = serde_json::from_str(data).unwrap();
        assert!(matches!(c.kind, DataType::String(_)));
    }

    #[test]
    fn handles_complex_string_with_constraints() {
        let data =
            r#"{ "type": { "key": { "type": "string", "minLength": 0, "maxLength": 32 } } }"#;
        let c: Column = serde_json::from_str(data).unwrap();
        assert!(matches!(c.kind, DataType::String(_)));
        if let DataType::String(constraints) = c.kind {
            assert_eq!(constraints.min, Some(0));
            assert_eq!(constraints.max, Some(32));
            assert_eq!(constraints.options, None);
        } else {
            panic!()
        }
    }

    #[test]
    fn handles_string_enum() {
        let data = r#"{ "type": { "key": { "type": "string", "enum": ["set", ["One", "Two", "Three"]] } } }"#;
        let c: Column = serde_json::from_str(data).unwrap();
        assert!(matches!(c.kind, DataType::String(_)));
        if let DataType::String(constraints) = c.kind {
            assert_eq!(constraints.min, None);
            assert_eq!(constraints.max, None);
            assert_eq!(
                constraints.options,
                Some(vec![
                    "One".to_string(),
                    "Two".to_string(),
                    "Three".to_string()
                ])
            );
        } else {
            panic!()
        }
    }

    #[test]
    fn handles_scalar_uuid() {
        let data = r#"{ "type": "uuid" }"#;
        let c: Column = serde_json::from_str(data).unwrap();
        assert!(matches!(c.kind, DataType::Uuid { .. }));
    }

    #[test]
    fn handles_complex_uuid() {
        let data = r#"{ "type": { "key": { "type": "uuid", "refTable": "other_table", "refType": "weak" } } }"#;
        let c: Column = serde_json::from_str(data).unwrap();
        if let DataType::Uuid {
            ref_table,
            ref_type,
        } = c.kind
        {
            assert_eq!(ref_table, Some("other_table".to_string()));
            assert_eq!(ref_type, Some(RefType::Weak));
        } else {
            panic!()
        }
    }

    #[test]
    fn handles_simple_map() {
        let data = r#"{ "type": { "key": "string", "value": "string" } }"#;
        let c: Column = serde_json::from_str(data).unwrap();
        if let DataType::Map { key, value } = c.kind {
            assert!(matches!(*key, DataType::String(_)));
            assert!(matches!(*value, DataType::String(_)));
        } else {
            panic!();
        }
    }

    #[test]
    fn handles_complex_map() {
        let data = r#"{ "type": { "key": { "type": "string", "enum": ["set", ["width", "height"]] }, "value": { "type": "string", "minLength": 5, "maxLength": 20 } } }"#;
        let c: Column = serde_json::from_str(data).unwrap();
        if let DataType::Map { key, value } = c.kind {
            if let DataType::String(constraints) = *key {
                assert_eq!(
                    constraints.options,
                    Some(vec!["width".to_string(), "height".to_string(),])
                );
            } else {
                panic!();
            }
            if let DataType::String(constraints) = *value {
                assert_eq!(constraints.min, Some(5));
                assert_eq!(constraints.max, Some(20));
                assert_eq!(constraints.options, None);
            } else {
                panic!();
            }
        } else {
            panic!();
        }
    }
}
