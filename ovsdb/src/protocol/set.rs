use std::marker::PhantomData;
use std::ops::Deref;

use serde::{
    de::{self, Deserializer, SeqAccess, Visitor},
    ser::{SerializeSeq, Serializer},
    Deserialize, Serialize,
};

use super::Uuid;

#[derive(Clone, Debug, PartialEq)]
pub struct Set<T>(pub Vec<T>);

impl<T> Deref for Set<T> {
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

impl<'de, T> From<Vec<T>> for Set<T>
where
    T: Serialize + Deserialize<'de>,
{
    fn from(value: Vec<T>) -> Self {
        Self(value)
    }
}

impl<'de, T> From<Set<T>> for Vec<T>
where
    T: Serialize + Deserialize<'de>,
{
    fn from(value: Set<T>) -> Self {
        value.0
    }
}

impl<'de, T> Deserialize<'de> for Set<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SetVisitor<T> {
            marker: PhantomData<fn() -> Set<T>>,
        }

        impl<T> SetVisitor<T> {
            fn new() -> Self {
                SetVisitor {
                    marker: PhantomData,
                }
            }
        }

        impl<'de, T> Visitor<'de> for SetVisitor<T>
        where
            T: Deserialize<'de>,
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

        deserializer.deserialize_seq(SetVisitor::new())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UuidSet(pub Vec<Uuid>);

impl Deref for UuidSet {
    type Target = Vec<Uuid>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Serialize for UuidSet {
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

impl From<Vec<Uuid>> for UuidSet {
    fn from(value: Vec<Uuid>) -> Self {
        Self(value)
    }
}

impl From<UuidSet> for Vec<Uuid> {
    fn from(value: UuidSet) -> Self {
        value.0
    }
}

impl<'de> Deserialize<'de> for UuidSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct UuidSetVisitor;

        impl<'de> Visitor<'de> for UuidSetVisitor {
            type Value = UuidSet;

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
                        let set: Vec<Uuid> = value.next_element()?.unwrap();
                        Ok(UuidSet(set))
                    }
                    "uuid" => {
                        let s: String = value.next_element()?.unwrap();
                        let uuid = ::uuid::Uuid::parse_str(&s).map_err(de::Error::custom)?;
                        Ok(UuidSet(vec![Uuid::from(uuid)]))
                    }
                    _ => Err(de::Error::invalid_value(
                        de::Unexpected::Str(&kind),
                        &"set or uuid",
                    )),
                }
            }
        }

        deserializer.deserialize_seq(UuidSetVisitor)
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
