use std::marker::PhantomData;
use std::ops::Deref;

use serde::{
    de::{self, Deserializer, SeqAccess, Visitor},
    ser::{SerializeSeq, Serializer},
    Deserialize, Serialize,
};

#[derive(Debug)]
pub struct Set<T>(pub Vec<T>)
where
    T: Serialize;

impl<T> Deref for Set<T>
where
    T: Serialize,
{
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Serialize for Set<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(2))?;
        seq.serialize_element("set")?;
        seq.serialize_element(&self.0)?;
        seq.end()
    }
}

struct UuidSetVisitor;
impl<'de> Visitor<'de> for UuidSetVisitor {
    type Value = Set<super::Uuid>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("`array`")
    }

    fn visit_seq<S>(self, mut value: S) -> Result<Self::Value, S::Error>
    where
        S: SeqAccess<'de>,
    {
        let kind: String = value.next_element()?.unwrap();
        match kind.as_str() {
            "set" => {
                let set: Vec<super::Uuid> = value.next_element()?.unwrap();
                Ok(Set(set))
            }
            "uuid" => {
                let s: String = value.next_element()?.unwrap();
                let uuid = ::uuid::Uuid::parse_str(&s).map_err(de::Error::custom)?;
                let set: Self::Value = Set(vec![super::Uuid::from(uuid)]);
                Ok(set)
            }
            _ => Err(de::Error::invalid_value(de::Unexpected::Str(&kind), &"set")),
        }
    }
}

struct SetVisitor<T>
where
    T: Serialize,
{
    marker: PhantomData<fn() -> Set<T>>,
}

impl<T> SetVisitor<T>
where
    T: Serialize,
{
    fn new() -> Self {
        SetVisitor {
            marker: PhantomData,
        }
    }
}

impl<'de, T> Visitor<'de> for SetVisitor<T>
where
    T: Deserialize<'de> + Serialize,
{
    type Value = Set<T>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("`array`")
    }

    fn visit_seq<S>(self, mut value: S) -> Result<Self::Value, S::Error>
    where
        S: SeqAccess<'de>,
    {
        let kind: String = value.next_element()?.unwrap();
        match kind.as_str() {
            "set" => {
                let set: Vec<T> = value.next_element()?.unwrap();
                Ok(Set(set))
            }
            _ => Err(de::Error::invalid_value(de::Unexpected::Str(&kind), &"set")),
        }
    }
}

impl<'de> Deserialize<'de> for Set<super::Uuid> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(UuidSetVisitor)
    }
}

impl<'de> Deserialize<'de> for Set<String> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(SetVisitor::new())
    }
}

impl<'de> Deserialize<'de> for Set<i32> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(SetVisitor::new())
    }
}

impl<'de> Deserialize<'de> for Set<i64> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(SetVisitor::new())
    }
}

impl<'de> Deserialize<'de> for Set<f64> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(SetVisitor::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Deserialize)]
    struct Foo {
        bar: Set<String>,
    }

    #[test]
    fn test_serialize() -> Result<(), serde_json::Error> {
        let expected = r#"["set",["red","blue"]]"#;
        let vec: Vec<String> = vec!["red".into(), "blue".into()];
        let value = Set(vec);
        let json = serde_json::to_string(&value)?;
        assert_eq!(json, expected);
        Ok(())
    }

    #[test]
    fn test_deserialize() -> Result<(), serde_json::Error> {
        let data = r#"{"bar": ["set",["red","blue"]]}"#;
        let foo: Foo = serde_json::from_str(&data)?;
        assert_eq!(foo.bar.first().unwrap(), &"red".to_string());
        assert_eq!(foo.bar.last().unwrap(), &"blue".to_string());
        Ok(())
    }
}
