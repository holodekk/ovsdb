use serde::{
    de::{self, DeserializeOwned, Deserializer},
    Deserialize,
};

use super::Set;

pub fn deserialize_enum<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + DeserializeOwned,
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
