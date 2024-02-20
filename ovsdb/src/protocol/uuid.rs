use std::ops::Deref;

use ::uuid::Uuid as _Uuid;
use serde::{
    de::{self, Deserializer, SeqAccess, Visitor},
    ser::{SerializeSeq, Serializer},
    Deserialize, Serialize,
};

/// A unique identifier, usually representing a single entity in OVSDB.
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub struct Uuid(_Uuid);

impl Default for Uuid {
    fn default() -> Self {
        Self::from(_Uuid::new_v4())
    }
}

impl From<_Uuid> for Uuid {
    fn from(value: _Uuid) -> Self {
        Self(value)
    }
}

impl Deref for Uuid {
    type Target = _Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Serialize for Uuid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(2))?;
        seq.serialize_element("uuid")?;
        seq.serialize_element(&self.0)?;
        seq.end()
    }
}

impl<'de> Deserialize<'de> for Uuid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct UuidVisitor;

        impl<'de> Visitor<'de> for UuidVisitor {
            type Value = Uuid;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("`array`")
            }

            fn visit_seq<S>(self, mut value: S) -> Result<Self::Value, S::Error>
            where
                S: SeqAccess<'de>,
            {
                match value.next_element::<String>()? {
                    Some(kind) => match kind.as_str() {
                        "uuid" => {
                            let s: String = value.next_element()?.expect("uuid value");
                            let uuid = _Uuid::parse_str(&s).map_err(de::Error::custom)?;
                            Ok(Uuid(uuid))
                        }
                        _ => Err(de::Error::invalid_value(
                            de::Unexpected::Str(&kind),
                            &"uuid",
                        )),
                    },
                    None => Err(de::Error::custom(
                        "`uuid` specified, but value not provided",
                    )),
                }
            }
        }

        deserializer.deserialize_seq(UuidVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize() -> Result<(), serde_json::Error> {
        let expected = r#"["uuid","36bef046-7da7-43a5-905a-c17899216fcb"]"#;
        let uuid = uuid::Uuid::parse_str("36bef046-7da7-43a5-905a-c17899216fcb").expect("uuid");
        let value = Uuid(uuid);
        let json = serde_json::to_string(&value)?;
        assert_eq!(json, expected);
        Ok(())
    }

    #[test]
    fn test_deserialize() -> Result<(), serde_json::Error> {
        let data = r#"["uuid","36bef046-7da7-43a5-905a-c17899216fcb"]"#;
        let uuid: Uuid = serde_json::from_str(data)?;
        assert_eq!(&uuid.to_string(), "36bef046-7da7-43a5-905a-c17899216fcb");
        Ok(())
    }
}
