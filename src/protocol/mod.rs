use std::collections::BTreeMap;

use serde::{de::Deserializer, ser::Serializer, Deserialize, Serialize};

pub(crate) mod codec;
mod request;
pub use request::*;
mod response;
pub use response::*;
mod value;
pub use value::*;

pub fn from_bool<S>(value: bool, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let v: Value = value.into();
    v.serialize(serializer)
}

pub fn from_i64<S>(value: i64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let v: Value = value.into();
    v.serialize(serializer)
}

pub fn from_f64<S>(value: f64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let v: Value = value.into();
    v.serialize(serializer)
}

pub fn from_str<S>(value: &str, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let v: Value = value.into();
    v.serialize(serializer)
}

pub fn from_string<S>(value: String, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let v: Value = value.into();
    v.serialize(serializer)
}

pub fn from_set<S, T, I>(value: T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Iterator<Item = I>,
    I: Into<Value>,
{
    let mut vec: Vec<Value> = vec![];
    value.into_iter().for_each(|v| vec.push(v.into()));
    let set = Set(vec);
    let v = Value::Atom(Atom::Set(set));
    v.serialize(serializer)
}

pub fn from_map<S, C>(value: C, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    C: Into<Map>,
{
    let v = Value::Atom(Atom::Map(value.into()));
    v.serialize(serializer)
}

pub fn from_uuid<S>(value: uuid::Uuid, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let v = Value::Atom(Atom::Uuid(Uuid(value)));
    v.serialize(serializer)
}

pub fn to_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    match Value::deserialize(deserializer)? {
        Value::Scalar(Scalar::Boolean(b)) => Ok(b),
        _ => Err(serde::de::Error::custom("Invalid input")),
    }
}

pub fn to_i64<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    match Value::deserialize(deserializer)? {
        Value::Scalar(Scalar::Integer(i)) => Ok(i),
        _ => Err(serde::de::Error::custom("Invalid input")),
    }
}

pub fn to_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    match Value::deserialize(deserializer)? {
        Value::Scalar(Scalar::Real(f)) => Ok(f),
        _ => Err(serde::de::Error::custom("Invalid input")),
    }
}

pub fn to_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    match Value::deserialize(deserializer)? {
        Value::Scalar(Scalar::String(s)) => Ok(s),
        _ => Err(serde::de::Error::custom("Invalid input")),
    }
}

pub fn to_map<'de, D, K, V>(deserializer: D) -> Result<BTreeMap<K, V>, D::Error>
where
    D: Deserializer<'de>,
    K: TryFrom<Scalar> + Ord,
    <K as TryFrom<Scalar>>::Error: std::fmt::Display,
    V: TryFrom<Value>,
    <V as TryFrom<Value>>::Error: std::fmt::Display,
{
    match Value::deserialize(deserializer)? {
        Value::Atom(Atom::Map(Map(m))) => {
            let mut map: BTreeMap<K, V> = BTreeMap::new();

            for (k, v) in m {
                let key: K = Scalar::from(k)
                    .try_into()
                    .map_err(serde::de::Error::custom)?;
                let value: V = V::try_from(v).map_err(serde::de::Error::custom)?;
                map.insert(key, value);
            }
            Ok(map)
        }
        _ => Err(serde::de::Error::custom("Invalid input")),
    }
}

pub fn to_set<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: TryFrom<Value>,
    <T as TryFrom<Value>>::Error: std::fmt::Display,
{
    match Value::deserialize(deserializer)? {
        Value::Atom(Atom::Set(Set(s))) => {
            let mut set: Vec<T> = vec![];
            for v in s {
                set.push(v.try_into().map_err(serde::de::Error::custom)?);
            }
            // s.into_iter().for_each(|v| set.push(v.into()));
            Ok(set)
        }
        _ => Err(serde::de::Error::custom("Invalid input")),
    }
}

pub fn to_uuid<'de, D>(deserializer: D) -> Result<::uuid::Uuid, D::Error>
where
    D: Deserializer<'de>,
{
    match Value::deserialize(deserializer)? {
        Value::Atom(Atom::Uuid(Uuid(u))) => Ok(u),
        _ => Err(serde::de::Error::custom("Invalid input")),
    }
}
