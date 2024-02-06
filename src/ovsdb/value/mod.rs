use std::convert::From;

use serde::{
    de::{self, Deserializer, MapAccess, SeqAccess, Visitor},
    ser::Serializer,
    Deserialize, Serialize,
};

mod atom;
pub use atom::*;
mod map;
pub use map::*;
mod scalar;
pub use scalar::*;
mod set;
pub use set::*;
mod uuid;
pub use self::uuid::*;

#[derive(Debug, PartialEq)]
pub enum Value {
    Scalar(Scalar),
    Atom(Atom),
}

impl From<&str> for Value {
    fn from(value: &str) -> Value {
        Value::Scalar(Scalar::from(value))
    }
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Value::Scalar(s) => s.serialize(serializer),
            Value::Atom(s) => s.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = Value;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("`bool`, `u64`, `i64`, `string`, `array`, `object`")
            }

            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Self::Value::Scalar(Scalar::from(value)))
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Self::Value::Scalar(Scalar::from(value)))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Self::Value::Scalar(Scalar::from(value)))
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Self::Value::Scalar(Scalar::from(value)))
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Self::Value::Scalar(Scalar::from(value)))
            }

            fn visit_seq<S>(self, value: S) -> Result<Self::Value, S::Error>
            where
                S: SeqAccess<'de>,
            {
                let atom = Atom::deserialize(de::value::SeqAccessDeserializer::new(value))?;
                Ok(Self::Value::Atom(atom))
            }

            fn visit_map<M>(self, value: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let map = Map::deserialize(de::value::MapAccessDeserializer::new(value))?;
                Ok(Self::Value::Atom(Atom::Map(map)))
            }
        }
        deserializer.deserialize_any(ValueVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_uuid() -> Result<(), serde_json::Error> {
        let expected = r#"["uuid","36bef046-7da7-43a5-905a-c17899216fcb"]"#;
        let value = Value::Atom(Atom::Uuid(
            Uuid::try_from("36bef046-7da7-43a5-905a-c17899216fcb").unwrap(),
        ));
        let json = serde_json::to_string(&value)?;
        assert_eq!(json, expected);
        Ok(())
    }

    #[test]
    fn deserialize_bool() -> Result<(), serde_json::Error> {
        let data = r#"true"#;
        let value: Value = serde_json::from_str(data)?;
        assert!(matches!(value, Value::Scalar(Scalar::Boolean(true))));
        Ok(())
    }

    #[test]
    fn deserialize_integer() -> Result<(), serde_json::Error> {
        let data = r#"123"#;
        let value: Value = serde_json::from_str(data)?;
        assert!(matches!(value, Value::Scalar(Scalar::Integer(123))));
        Ok(())
    }

    #[test]
    fn deserialize_real() -> Result<(), serde_json::Error> {
        let data = r#"123.456"#;
        let value: Value = serde_json::from_str(data)?;
        match value {
            Value::Scalar(Scalar::Real(r)) => assert_eq!(r, 123.456),
            _ => panic!("Invalid atom derived from float: {:#?}", value),
        }
        Ok(())
    }

    #[test]
    fn deserialize_string() -> Result<(), serde_json::Error> {
        let data = r#""hello""#;
        let value: Value = serde_json::from_str(data)?;
        match value {
            Value::Scalar(Scalar::String(s)) => assert_eq!(s, "hello"),
            _ => panic!("Invalid atom derived from string: {:#?}", value),
        }
        Ok(())
    }

    #[test]
    fn deserialize_map() -> Result<(), serde_json::Error> {
        let data = r#"["map", {"color": "blue"}]"#;
        let value: Value = serde_json::from_str(data)?;
        match value {
            Value::Atom(Atom::Map(m)) => {
                assert_eq!(m.get("color").unwrap(), &Scalar::from("blue"));
            }
            _ => panic!("Invalid value for map: {:#?}", value),
        }
        Ok(())
    }

    #[test]
    fn deserialize_set() -> Result<(), serde_json::Error> {
        let data = r#"["set", ["one", "two", "three"]]"#;
        let value: Value = serde_json::from_str(data)?;
        match value {
            Value::Atom(Atom::Set(s)) => {
                assert_eq!(
                    s,
                    Set(vec![
                        Value::Scalar(Scalar::from("one")),
                        Value::Scalar(Scalar::from("two")),
                        Value::Scalar(Scalar::from("three"))
                    ])
                );
            }
            _ => panic!("Invalid atom for map: {:#?}", value),
        }
        Ok(())
    }

    #[test]
    fn deserialize_uuid() -> Result<(), serde_json::Error> {
        let data = r#"["uuid", "36bef046-7da7-43a5-905a-c17899216fcb"]"#;
        let value: Value = serde_json::from_str(data)?;
        match value {
            Value::Atom(Atom::Uuid(u)) => {
                assert_eq!(u.to_string(), "36bef046-7da7-43a5-905a-c17899216fcb")
            }
            _ => panic!("Invalid atom for uuid: {:#?}", value),
        }
        Ok(())
    }
}
