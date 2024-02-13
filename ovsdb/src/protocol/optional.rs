use serde::{
    de::{self, DeserializeOwned, Deserializer},
    ser::{SerializeSeq, Serializer},
    Deserialize, Serialize,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Optional<T>(Option<T>);

impl<'de, T> From<Option<T>> for Optional<T>
where
    T: Deserialize<'de> + Serialize,
{
    fn from(value: Option<T>) -> Self {
        Self(value)
    }
}

impl<'de, T> From<Optional<T>> for Option<T>
where
    T: Deserialize<'de> + Serialize,
{
    fn from(value: Optional<T>) -> Self {
        value.0
    }
}

impl<T> Serialize for Optional<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if self.0.is_some() {
            self.0.serialize(serializer)
        } else {
            let mut seq = serializer.serialize_seq(Some(2))?;
            seq.serialize_element("map")?;
            let set: Vec<i32> = vec![];
            seq.serialize_element(&set)?;
            seq.end()
        }
    }
}

impl<'de, T> Deserialize<'de> for Optional<T>
where
    T: DeserializeOwned,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v = serde_json::Value::deserialize(deserializer)?;

        // Check for an empty set (indicates an optional value)
        if let Some(arr) = v.as_array() {
            if arr.len() == 2 {
                if let [k, v] = arr.as_slice() {
                    if k.as_str() == Some("set") {
                        if let Some(inner) = v.as_array() {
                            if inner.is_empty() {
                                return Ok(Optional(None));
                            }
                        }
                    }
                }
            }
        }

        // Force a deserialize to the target type (will either work or throw an actionable error
        let target: T = serde_json::from_value(v).map_err(de::Error::custom)?;
        Ok(Optional(Some(target)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optional_string() {
        #[derive(Deserialize)]
        struct Test {
            foo: Optional<String>,
        }

        let data = r#"{"foo": ["set", []]}"#;
        let value: Test = serde_json::from_str(&data).unwrap();
        assert_eq!(value.foo, Optional(None));
    }

    #[test]
    fn test_optional_uuid_none() {
        #[derive(Deserialize)]
        struct Test {
            foo: Optional<crate::protocol::Uuid>,
        }

        let data = r#"{"foo": ["set", []]}"#;
        let value: Test = serde_json::from_str(&data).unwrap();
        assert_eq!(value.foo, Optional(None));
    }

    #[test]
    fn test_optional_uuid_some() {
        #[derive(Deserialize)]
        struct Test {
            foo: Optional<crate::protocol::Uuid>,
        }

        let data = r#"{"foo": ["uuid", "06234b93-6b4b-4f92-be8a-342dd858617c"]}"#;
        let value: Test = serde_json::from_str(&data).unwrap();
        assert!(matches!(value.foo, Optional(Some(_))));
    }
}
