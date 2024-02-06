use std::convert::From;

use serde::{
    de::{self, Deserializer, Visitor},
    ser::Serializer,
    Deserialize, Serialize,
};

/// Represents a scalar value within ovsdb.
#[derive(Debug, PartialEq)]
pub enum Scalar {
    /// Raw boolean value
    Boolean(bool),
    /// Raw integer value
    Integer(i64),
    /// Raw float value
    Real(f64),
    /// Raw string value
    String(String),
}

impl From<bool> for Scalar {
    fn from(value: bool) -> Scalar {
        Scalar::Boolean(value)
    }
}

impl From<i32> for Scalar {
    fn from(value: i32) -> Scalar {
        Scalar::Integer(value.try_into().unwrap())
    }
}

impl From<i64> for Scalar {
    fn from(value: i64) -> Scalar {
        Scalar::Integer(value)
    }
}

impl From<u64> for Scalar {
    fn from(value: u64) -> Scalar {
        Scalar::Integer(value.try_into().unwrap())
    }
}

impl From<f64> for Scalar {
    fn from(value: f64) -> Scalar {
        Scalar::Real(value)
    }
}

impl From<&str> for Scalar {
    fn from(value: &str) -> Scalar {
        Scalar::String(value.to_string())
    }
}

impl From<String> for Scalar {
    fn from(value: String) -> Scalar {
        Scalar::String(value)
    }
}

impl From<&String> for Scalar {
    fn from(value: &String) -> Scalar {
        Scalar::String(value.clone())
    }
}

impl Serialize for Scalar {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Scalar::Boolean(v) => serializer.serialize_bool(*v),
            Scalar::Integer(v) => serializer.serialize_i64(*v),
            Scalar::Real(v) => serializer.serialize_f64(*v),
            Scalar::String(v) => serializer.serialize_str(v),
        }
    }
}

impl<'de> Deserialize<'de> for Scalar {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ScalarVisitor;

        impl<'de> Visitor<'de> for ScalarVisitor {
            type Value = Scalar;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("`bool`, `i64`, `f64` or `string`")
            }

            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Self::Value::from(value))
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Self::Value::from(value))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Self::Value::from(value))
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Self::Value::from(value))
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Self::Value::from(value))
            }
        }

        deserializer.deserialize_any(ScalarVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_bool() -> Result<(), serde_json::Error> {
        let expected = r#"true"#;
        let value = Scalar::from(true);
        let json = serde_json::to_string(&value)?;
        assert_eq!(json, expected);
        Ok(())
    }

    #[test]
    fn serialize_i64() -> Result<(), serde_json::Error> {
        let expected = r#"123"#;
        let value = Scalar::from(123);
        let json = serde_json::to_string(&value)?;
        assert_eq!(json, expected);
        Ok(())
    }

    #[test]
    fn serialize_f64() -> Result<(), serde_json::Error> {
        let expected = r#"123.456"#;
        let value = Scalar::from(123.456);
        let json = serde_json::to_string(&value)?;
        assert_eq!(json, expected);
        Ok(())
    }

    #[test]
    fn serialize_string() -> Result<(), serde_json::Error> {
        let expected = r#""hello""#;
        let value = Scalar::from("hello");
        let json = serde_json::to_string(&value)?;
        assert_eq!(json, expected);
        Ok(())
    }

    #[test]
    fn deserialize_bool() -> Result<(), serde_json::Error> {
        let value = r#"true"#;
        let scalar: Scalar = serde_json::from_str(&value)?;
        assert!(matches!(scalar, Scalar::Boolean(true)));
        Ok(())
    }

    #[test]
    fn deserialize_i64() -> Result<(), serde_json::Error> {
        let value = r#"123"#;
        let scalar: Scalar = serde_json::from_str(&value)?;
        assert!(matches!(scalar, Scalar::Integer(123)));
        Ok(())
    }

    #[test]
    fn deserialize_f64() -> Result<(), serde_json::Error> {
        let value = r#"123.456"#;
        let scalar: Scalar = serde_json::from_str(&value)?;
        assert!(matches!(scalar, Scalar::Real(_)));
        match scalar {
            Scalar::Real(f) => assert_eq!(f, 123.456),
            _ => unreachable!(),
        }
        Ok(())
    }

    #[test]
    fn deserialize_string() -> Result<(), serde_json::Error> {
        let value = r#""teststring""#;
        let scalar: Scalar = serde_json::from_str(&value)?;
        assert!(matches!(scalar, Scalar::String(_)));
        match scalar {
            Scalar::String(s) => assert_eq!(&s, "teststring"),
            _ => unreachable!(),
        }
        Ok(())
    }

    #[test]
    #[should_panic]
    fn deserialize_invalid() {
        let value = r#"{"an": "object"}"#;
        let _ = serde_json::from_str::<Scalar>(&value).unwrap();
    }
}
