use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use serde::Deserialize;
use serde_json::Value;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub enum RefType {
    #[serde(rename = "strong")]
    Strong,
    #[serde(rename = "weak")]
    Weak,
}

fn extract_options<'a, T>(c: &'a Option<&'a Value>) -> Result<Option<Vec<T>>, serde_json::Error>
where
    T: Deserialize<'a>,
{
    if let Some(o) = c {
        if o.is_array() {
            let s = o.as_array().unwrap();
            assert_eq!(s.len(), 2);

            if s[0].as_str().unwrap() == "set" {
                let values: Vec<T> = s[1]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| T::deserialize(v).unwrap())
                    .collect();
                return Ok(Some(values));
            }
        }
    }
    Ok(None)
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Constraints<T, O> {
    pub min: Option<T>,
    pub max: Option<T>,
    pub options: Option<Vec<O>>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DataType {
    Boolean,
    Integer(Constraints<i64, i64>),
    Real(Constraints<f64, f64>),
    String(Constraints<i64, String>),
    Uuid {
        ref_table: Option<String>,
        ref_type: Option<RefType>,
    },
    Map {
        key: Box<DataType>,
        value: Box<DataType>,
    },
    Unknown,
}

impl DataType {
    pub fn is_enum(&self) -> bool {
        match self {
            Self::Integer(c) => c.options.is_some(),
            Self::Real(c) => c.options.is_some(),
            Self::String(c) => c.options.is_some(),
            _ => false,
        }
    }

    pub fn from_value(data: &Value) -> Result<Self, serde_json::Error> {
        let kind = match data {
            Value::String(s) => match s.as_str() {
                "boolean" => Self::Boolean,
                "integer" => Self::Integer(Constraints::default()),
                "real" => Self::Real(Constraints::default()),
                "string" => Self::String(Constraints::default()),
                "uuid" => Self::Uuid {
                    ref_table: None,
                    ref_type: None,
                },
                _ => Self::Unknown,
            },
            Value::Object(o) => {
                let type_obj = o.get("type").unwrap();
                match type_obj {
                    Value::String(s) => match s.as_str() {
                        "boolean" => Self::Boolean,
                        "integer" => Self::Integer(Constraints {
                            options: extract_options(&o.get("enum")).unwrap(),
                            min: o.get("minInteger").map(|v| v.as_i64().unwrap()),
                            max: o.get("maxInteger").map(|v| v.as_i64().unwrap()),
                        }),
                        "real" => Self::Real(Constraints {
                            options: extract_options(&o.get("enum")).unwrap(),
                            min: o.get("minReal").map(|v| v.as_f64().unwrap()),
                            max: o.get("maxReal").map(|v| v.as_f64().unwrap()),
                        }),
                        "string" => Self::String(Constraints {
                            options: extract_options(&o.get("enum")).unwrap(),
                            min: o.get("minLength").map(|v| v.as_i64().unwrap()),
                            max: o.get("maxLength").map(|v| v.as_i64().unwrap()),
                        }),
                        "uuid" => Self::Uuid {
                            ref_table: o.get("refTable").map(|v| v.as_str().unwrap().to_string()),
                            ref_type: o.get("refType").map(|t| RefType::deserialize(t).unwrap()),
                        },
                        _ => Self::Unknown,
                    },
                    Value::Object(typ) => {
                        let key = Self::from_value(typ.get("key").unwrap()).unwrap();
                        if typ.contains_key("value") {
                            let value = Self::from_value(typ.get("value").unwrap()).unwrap();
                            Self::Map {
                                key: Box::new(key),
                                value: Box::new(value),
                            }
                        } else {
                            key
                        }
                    }
                    _ => Self::Unknown,
                }
            }
            _ => Self::Unknown,
        };

        Ok(kind)
    }
}

impl ToTokens for DataType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            DataType::Boolean => {
                tokens.append(format_ident!("{}", "bool"));
            }
            DataType::Integer(_) => {
                tokens.append(format_ident!("{}", "i64"));
            }
            DataType::Real(_) => {
                tokens.append(format_ident!("{}", "f64"));
            }
            DataType::String(_) => {
                tokens.append(format_ident!("{}", "String"));
            }
            DataType::Uuid { .. } => {
                tokens.extend(quote! {
                    uuid::Uuid
                });
            }
            DataType::Map { key, value } => {
                let key_type = key.to_token_stream();
                let value_type = value.to_token_stream();
                tokens.extend(quote! {
                    BTreeMap<#key_type, #value_type>
                });
            }
            _ => unreachable!(),
        }
    }
}
