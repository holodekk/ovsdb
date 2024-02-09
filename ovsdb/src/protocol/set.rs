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

impl<'de, T> Deserialize<'de> for Set<T>
where
    T: Deserialize<'de> + Serialize,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
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
