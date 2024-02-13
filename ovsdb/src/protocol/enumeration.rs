use serde::{
    de::{self, DeserializeOwned, Deserializer},
    ser::Serializer,
    Deserialize, Serialize,
};

use super::Set;

pub fn serialize<T, S>(v: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: Serialize + PartialEq,
    S: Serializer,
{
    if v == &T::default() {
        Set::<i32>(vec![]).serialize(serializer)
    } else {
        v.serialize(serializer)
    }
}

pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned,
{
    let value = serde_json::Value::deserialize(deserializer)?;

    if value.is_string() {
        serde_json::from_value::<T>(value).map_err(de::Error::custom)
    } else {
        match serde_json::from_value::<Set<i32>>(value) {
            Ok(set) => {
                if set.is_empty() {
                    Ok(T::default())
                } else {
                    Err(de::Error::custom(format!(
                        "expected empty set for enum, found `{:#?}`",
                        set,
                    )))
                }
            }
            Err(e) => Err(de::Error::custom(e)),
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_serialize() -> Result<(), serde_json::Error> {
//         let expected = r#"["map",[["color","blue"]]]"#;
//         let mut map: BTreeMap<String, String> = BTreeMap::new();
//         map.insert("color".to_string(), "blue".to_string());
//         let value = Map(map);
//         let json = serde_json::to_string(&value)?;
//         assert_eq!(json, expected);
//         Ok(())
//     }

//     #[test]
//     fn test_deserialize() -> Result<(), serde_json::Error> {
//         let data = r#"["map",[["color","blue"]]]"#;
//         let map: Map<String, String> = serde_json::from_str(&data)?;
//         assert_eq!(map.get("color").unwrap(), "blue");
//         Ok(())
//     }
// }
