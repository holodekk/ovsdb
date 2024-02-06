use std::convert::{AsRef, TryFrom};
use std::ops::Deref;

use ::uuid::Uuid as _Uuid;
use serde::{
    de::{self, Deserializer, Visitor},
    ser::Serializer,
    Deserialize, Serialize,
};

/// A representation of a UUID value within OVSDB.
#[derive(Debug, PartialEq)]
pub struct Uuid(_Uuid);

impl Uuid {
    /// generates a new random Uuid
    pub fn generate() -> Self {
        Self(_Uuid::new_v4())
    }
}

impl ToString for Uuid {
    fn to_string(&self) -> String {
        self.0
            .as_hyphenated()
            .encode_lower(&mut _Uuid::encode_buffer())
            .to_string()
    }
}

impl TryFrom<&str> for Uuid {
    type Error = uuid::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let u = uuid::Uuid::parse_str(value)?;
        Ok(Self(u))
    }
}

impl TryFrom<String> for Uuid {
    type Error = uuid::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Uuid::try_from(value.as_str())
    }
}

impl Deref for Uuid {
    type Target = _Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> AsRef<T> for Uuid
where
    T: ?Sized,
    <Uuid as Deref>::Target: AsRef<T>,
{
    fn as_ref(&self) -> &T {
        self.deref().as_ref()
    }
}

impl Serialize for Uuid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Uuid {
    fn deserialize<D>(deserializer: D) -> Result<Uuid, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct UuidVisitor;

        impl<'de> Visitor<'de> for UuidVisitor {
            type Value = Uuid;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("`string`")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Uuid(_Uuid::parse_str(value).map_err(de::Error::custom)?))
            }
        }

        deserializer.deserialize_str(UuidVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize() -> Result<(), serde_json::Error> {
        let expected = r#""36bef046-7da7-43a5-905a-c17899216fcb""#;
        let value = Uuid::try_from("36bef046-7da7-43a5-905a-c17899216fcb").unwrap();
        let json = serde_json::to_string(&value)?;
        assert_eq!(json, expected);
        Ok(())
    }

    #[test]
    fn deserialize() -> Result<(), serde_json::Error> {
        let uuid_str = "36bef046-7da7-43a5-905a-c17899216fcb";
        let serialized = serde_json::to_string(uuid_str)?;
        let uuid: Uuid = serde_json::from_str(&serialized)?;
        assert_eq!(&uuid.to_string(), uuid_str);
        Ok(())
    }
}
