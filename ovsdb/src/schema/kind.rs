use std::str::FromStr;

use serde::{
    de::{self, Deserializer, MapAccess, Visitor},
    Deserialize, Serialize,
};

use crate::protocol::Set;

use super::Atomic;

/// A reference to another object
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum RefType {
    /// A strong reference (one that will always point to a valid entity)
    #[serde(rename = "strong")]
    Strong,
    /// A weak reference (one whose target may not exist)
    #[serde(rename = "weak")]
    Weak,
}

/// The most basic atomic type in OVSDB.
///
/// Includes optional constraints which control the values allowed in the [Column][super::Column].
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct BaseKind {
    kind: Atomic,
    choices: Option<Set<String>>,
    min_integer: Option<i64>,
    max_integer: Option<i64>,
    min_real: Option<f64>,
    max_real: Option<f64>,
    min_length: Option<i64>,
    max_length: Option<i64>,
    ref_table: Option<String>,
    ref_type: Option<RefType>,
}

impl BaseKind {
    #[must_use]
    pub(crate) fn new(kind: Atomic) -> Self {
        Self {
            kind,
            ..BaseKind::default()
        }
    }

    /// The atomic kind stored in this [Column][super::Column].
    #[must_use]
    pub fn kind(&self) -> Atomic {
        self.kind
    }

    /// If this [Column][super::Column] is an enumeration, returns the allowed values.
    #[must_use]
    pub fn choices(&self) -> Option<&Set<String>> {
        self.choices.as_ref()
    }

    /// If this is an [Integer][Atomic::Integer] [Column][super::Column], the minimum value allowed.
    #[must_use]
    pub fn min_integer(&self) -> Option<&i64> {
        self.min_integer.as_ref()
    }

    /// If this is an [Integer][Atomic::Integer] [Column][super::Column], the maximum value allowed.
    #[must_use]
    pub fn max_integer(&self) -> Option<&i64> {
        self.max_integer.as_ref()
    }

    /// If this is an [Real][Atomic::Real] [Column][super::Column], the minimum value allowed.
    #[must_use]
    pub fn min_real(&self) -> Option<&f64> {
        self.min_real.as_ref()
    }

    /// If this is an [Real][Atomic::Real] [Column][super::Column], the maximum value allowed.
    #[must_use]
    pub fn max_real(&self) -> Option<&f64> {
        self.max_real.as_ref()
    }

    /// If this is an [String][Atomic::String] [Column][super::Column], the minimum length of the string.
    #[must_use]
    pub fn min_length(&self) -> Option<&i64> {
        self.min_length.as_ref()
    }

    /// If this is an [String][Atomic::String] [Column][super::Column], the maximum length of the string.
    #[must_use]
    pub fn max_length(&self) -> Option<&i64> {
        self.max_length.as_ref()
    }

    /// If this is a referential [Column][super::Column], the table the reference points to.
    #[must_use]
    pub fn ref_table(&self) -> Option<&str> {
        self.ref_table.as_deref()
    }

    /// If this is a referential [Column][super::Column], the type of reference represented.
    #[must_use]
    pub fn ref_type(&self) -> Option<RefType> {
        self.ref_type
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

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

/// Represents the type of a database [Column][super::Column].
#[derive(Clone, Debug, Serialize)]
pub struct Kind {
    key: BaseKind,
    /// If present, represents the type of the value for a map type column.
    value: Option<BaseKind>,
    /// Minimum number of values allowed.
    min: i64,
    /// Maximum number of values allowed.
    max: i64,
}

impl Kind {
    #[must_use]
    pub(crate) fn new(key: BaseKind) -> Self {
        Self {
            key,
            ..Self::default()
        }
    }

    /// Either the raw value stored in the [Column][super::Column], or the type of key for a map type column.
    #[must_use]
    pub fn key(&self) -> &BaseKind {
        &self.key
    }

    /// The type associated with the value in a map column.
    #[must_use]
    pub fn value(&self) -> Option<&BaseKind> {
        self.value.as_ref()
    }

    /// Returs true if this is a simple scalar value.
    #[must_use]
    pub fn is_scalar(&self) -> bool {
        self.value.is_none() && self.min == 1 && self.max == 1
    }

    /// Returns true if this is an optional scalar value.
    #[must_use]
    pub fn is_optional(&self) -> bool {
        self.min == 0 && self.max == 1
    }

    /// Returns true if this is a set or map type.
    #[must_use]
    pub fn is_composite(&self) -> bool {
        self.max > 1
    }

    /// Returns true if this is a set type.
    #[must_use]
    pub fn is_set(&self) -> bool {
        self.value.is_none() && (self.min != 1 || self.max != 1)
    }

    /// Returns true if this is a map type.
    #[must_use]
    pub fn is_map(&self) -> bool {
        self.value.is_some()
    }

    /// Returns true if this is an enumeration.
    #[must_use]
    pub fn is_enum(&self) -> bool {
        self.value.is_none() && self.key.choices.is_some()
    }

    /// Returns true if this is an optional pointer to another table record.
    #[must_use]
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

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
        let k: BaseKind = serde_json::from_str(data).expect("BaseKind");
        assert_eq!(k.kind, Atomic::Integer);
    }

    #[test]
    fn test_base_kind_integer_key() {
        let data = r#"{"type": "integer"}"#;
        let k: BaseKind = serde_json::from_str(data).expect("BaseKind");
        assert_eq!(k.kind, Atomic::Integer);
    }

    #[test]
    fn test_base_kind_integer_full() {
        let data = r#"{"type": "integer", "minInteger": 1, "maxInteger": 100}"#;
        let k: BaseKind = serde_json::from_str(data).expect("BaseKind");
        assert_eq!(k.kind, Atomic::Integer);
        assert_eq!(k.min_integer, Some(1));
        assert_eq!(k.max_integer, Some(100));
    }

    #[test]
    fn test_kind_string() {
        let data = r#""boolean""#;
        let k: Kind = serde_json::from_str(data).expect("Kind");
        assert_eq!(k.key.kind, Atomic::Boolean);
        assert_eq!(k.value, None);
        assert_eq!(k.min, 1);
        assert_eq!(k.max, 1);
    }

    #[test]
    fn test_kind_simple_key() {
        let data = r#"{"key": "boolean"}"#;
        let k: Kind = serde_json::from_str(data).expect("Kind");
        assert_eq!(k.key.kind, Atomic::Boolean);
        assert_eq!(k.value, None);
        assert_eq!(k.min, 1);
        assert_eq!(k.max, 1);
    }

    #[test]
    fn test_kind_complex_key() {
        let data = r#"{"key": {"type": "boolean"}, "min": 1, "max": 100}"#;
        let k: Kind = serde_json::from_str(data).expect("Kind");
        assert_eq!(k.key.kind, Atomic::Boolean);
        assert_eq!(k.value, None);
        assert_eq!(k.min, 1);
        assert_eq!(k.max, 100);
    }
}
