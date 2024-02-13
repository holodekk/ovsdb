use std::str::FromStr;

use serde::{
    de::{self, Deserializer, MapAccess, Visitor},
    Deserialize,
};

use crate::protocol::Set;

use super::Atomic;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub enum RefType {
    #[serde(rename = "strong")]
    Strong,
    #[serde(rename = "weak")]
    Weak,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BaseKind {
    pub kind: Atomic,
    pub choices: Option<Set<String>>,
    pub min_integer: Option<i64>,
    pub max_integer: Option<i64>,
    pub min_real: Option<f64>,
    pub max_real: Option<f64>,
    pub min_length: Option<i64>,
    pub max_length: Option<i64>,
    pub ref_table: Option<String>,
    pub ref_type: Option<RefType>,
}

impl BaseKind {
    pub fn new(kind: Atomic) -> Self {
        Self {
            kind,
            ..BaseKind::default()
        }
    }
}

impl<'de> Deserialize<'de> for BaseKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct BaseKindVisitor;

        impl<'de> Visitor<'de> for BaseKindVisitor {
            type Value = BaseKind;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("`map` or `string`")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match Atomic::from_str(value) {
                    Ok(a) => Ok(Self::Value::new(a)),
                    Err(_) => Err(de::Error::invalid_value(
                        de::Unexpected::Str(value),
                        &"one of `boolean`, `integer`, `real`, `string`, `uuid`",
                    )),
                }
            }

            fn visit_map<S>(self, mut value: S) -> Result<Self::Value, S::Error>
            where
                S: MapAccess<'de>,
            {
                let mut base = BaseKind::default();

                while let Some((k, v)) = value.next_entry::<String, serde_json::Value>()? {
                    match k.as_str() {
                        "type" => {
                            base.kind = serde_json::from_value(v).map_err(de::Error::custom)?
                        }
                        "enum" => {
                            base.choices =
                                Some(serde_json::from_value(v).map_err(de::Error::custom)?)
                        }
                        "minInteger" => {
                            base.min_integer =
                                Some(serde_json::from_value(v).map_err(de::Error::custom)?)
                        }
                        "maxInteger" => {
                            base.max_integer =
                                Some(serde_json::from_value(v).map_err(de::Error::custom)?)
                        }
                        "minReal" => {
                            base.min_real =
                                Some(serde_json::from_value(v).map_err(de::Error::custom)?)
                        }
                        "maxReal" => {
                            base.max_real =
                                Some(serde_json::from_value(v).map_err(de::Error::custom)?)
                        }
                        "minLength" => {
                            base.min_length =
                                Some(serde_json::from_value(v).map_err(de::Error::custom)?)
                        }
                        "maxLength" => {
                            base.max_length =
                                Some(serde_json::from_value(v).map_err(de::Error::custom)?)
                        }
                        "refTable" => {
                            base.ref_table =
                                Some(serde_json::from_value(v).map_err(de::Error::custom)?)
                        }
                        "refType" => {
                            base.ref_type =
                                Some(serde_json::from_value(v).map_err(de::Error::custom)?)
                        }
                        _ => Err(de::Error::unknown_field(
                            &k,
                            &[
                                "type",
                                "enum",
                                "minInteger",
                                "maxInteger",
                                "minReal",
                                "maxReal",
                                "minLength",
                                "maxLength",
                                "refTable",
                                "refType",
                            ],
                        ))?,
                    }
                }

                Ok(base)
            }
        }

        deserializer.deserialize_any(BaseKindVisitor)
    }
}

#[derive(Clone, Debug)]
pub struct Kind {
    pub key: BaseKind,
    pub value: Option<BaseKind>,
    pub min: i64,
    pub max: i64,
}

impl Kind {
    pub fn new(key: BaseKind) -> Self {
        Self {
            key,
            ..Self::default()
        }
    }

    pub fn is_scalar(&self) -> bool {
        self.value.is_none() && self.min == 1 && self.max == 1
    }

    pub fn is_optional(&self) -> bool {
        self.min == 0 && self.max == 1
    }

    pub fn is_composite(&self) -> bool {
        self.max > 1
    }

    pub fn is_set(&self) -> bool {
        self.value.is_none() && (self.min != 1 || self.max != 1)
    }

    pub fn is_map(&self) -> bool {
        self.value.is_some()
    }

    pub fn is_enum(&self) -> bool {
        self.value.is_none() && self.key.choices.is_some()
    }

    pub fn is_optional_pointer(&self) -> bool {
        self.is_optional()
            && self.value.is_none()
            && (self.key.kind == Atomic::String || self.key.ref_table.is_some())
    }
}

impl Default for Kind {
    fn default() -> Self {
        Self {
            key: BaseKind::default(),
            value: None,
            min: 1,
            max: 1,
        }
    }
}

impl<'de> Deserialize<'de> for Kind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct KindVisitor;

        impl<'de> Visitor<'de> for KindVisitor {
            type Value = Kind;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("`map` or `string`")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match Atomic::from_str(value) {
                    Ok(a) => Ok(Self::Value::new(BaseKind::new(a))),
                    Err(_) => Err(de::Error::invalid_value(
                        de::Unexpected::Str(value),
                        &"one of `boolean`, `integer`, `real`, `string`, `uuid`",
                    )),
                }
            }

            fn visit_map<S>(self, mut obj: S) -> Result<Self::Value, S::Error>
            where
                S: MapAccess<'de>,
            {
                let mut key: Option<BaseKind> = None;
                let mut value: Option<BaseKind> = None;
                let mut min: i64 = 1;
                let mut max: i64 = 1;

                while let Some((k, v)) = obj.next_entry::<String, serde_json::Value>()? {
                    match k.as_str() {
                        "key" => key = serde_json::from_value(v).map_err(de::Error::custom)?,
                        "value" => {
                            value = Some(serde_json::from_value(v).map_err(de::Error::custom)?)
                        }
                        "min" => min = serde_json::from_value(v).map_err(de::Error::custom)?,
                        "max" => {
                            max = if v.is_string() {
                                if v.as_str() == Some("unlimited") {
                                    -1
                                } else {
                                    // BIG ERROR
                                    todo!()
                                }
                            } else {
                                serde_json::from_value(v).map_err(de::Error::custom)?
                            }
                        }
                        _ => Err(de::Error::unknown_field(
                            &k,
                            &["key", "value", "min", "max"],
                        ))?,
                    }
                }

                if let Some(k) = key {
                    Ok(Self::Value {
                        key: k,
                        value,
                        min,
                        max,
                    })
                } else {
                    Err(de::Error::missing_field("key"))
                }
            }
        }

        deserializer.deserialize_any(KindVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_kind_integer_string() {
        let data = r#""integer""#;
        let k: BaseKind = serde_json::from_str(data).unwrap();
        assert_eq!(k.kind, Atomic::Integer);
    }

    #[test]
    fn test_base_kind_integer_key() {
        let data = r#"{"type": "integer"}"#;
        let k: BaseKind = serde_json::from_str(data).unwrap();
        assert_eq!(k.kind, Atomic::Integer);
    }

    #[test]
    fn test_base_kind_integer_full() {
        let data = r#"{"type": "integer", "minInteger": 1, "maxInteger": 100}"#;
        let k: BaseKind = serde_json::from_str(data).unwrap();
        assert_eq!(k.kind, Atomic::Integer);
        assert_eq!(k.min_integer, Some(1));
        assert_eq!(k.max_integer, Some(100));
    }

    #[test]
    fn test_kind_string() {
        let data = r#""boolean""#;
        let k: Kind = serde_json::from_str(data).unwrap();
        assert_eq!(k.key.kind, Atomic::Boolean);
        assert_eq!(k.value, None);
        assert_eq!(k.min, 1);
        assert_eq!(k.max, 1);
    }

    #[test]
    fn test_kind_simple_key() {
        let data = r#"{"key": "boolean"}"#;
        let k: Kind = serde_json::from_str(data).unwrap();
        assert_eq!(k.key.kind, Atomic::Boolean);
        assert_eq!(k.value, None);
        assert_eq!(k.min, 1);
        assert_eq!(k.max, 1);
    }

    #[test]
    fn test_kind_complex_key() {
        let data = r#"{"key": {"type": "boolean"}, "min": 1, "max": 100}"#;
        let k: Kind = serde_json::from_str(data).unwrap();
        assert_eq!(k.key.kind, Atomic::Boolean);
        assert_eq!(k.value, None);
        assert_eq!(k.min, 1);
        assert_eq!(k.max, 100);
    }
}
