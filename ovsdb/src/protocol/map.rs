use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::ops::Deref;

use serde::{
    de::{self, Deserializer, SeqAccess, Visitor},
    ser::{SerializeSeq, Serializer},
    Deserialize, Serialize,
};

/// Rust representation of the OVSDB `map` data type.
///
/// The OVSDB `map` is a dictionary type, containing key/value pairs.  The `map` itself is
/// represented on the wire as a tuple:
///
/// ```json
/// ["map", {"key": "value"}]
/// ```
#[derive(Clone, Debug)]
pub struct Map<K, V>(BTreeMap<K, V>)
where
    K: Serialize,
    V: Serialize;

impl<K, V> Deref for Map<K, V>
where
    K: Serialize,
    V: Serialize,
{
    type Target = BTreeMap<K, V>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de, K, V> From<BTreeMap<K, V>> for Map<K, V>
where
    K: Deserialize<'de> + Serialize,
    V: Deserialize<'de> + Serialize,
{
    fn from(value: BTreeMap<K, V>) -> Self {
        Map(value)
    }
}

impl<'de, K, V> From<Map<K, V>> for BTreeMap<K, V>
where
    K: Deserialize<'de> + Serialize,
    V: Deserialize<'de> + Serialize,
{
    fn from(value: Map<K, V>) -> Self {
        value.0
    }
}

impl<K, V> Serialize for Map<K, V>
where
    K: Serialize,
    V: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(2))?;
        seq.serialize_element("map")?;
        let mut map: Vec<(&K, &V)> = vec![];
        for (k, v) in &self.0 {
            map.push((k, v));
        }
        seq.serialize_element(&map)?;
        seq.end()
    }
}

impl<'de, K, V> Deserialize<'de> for Map<K, V>
where
    K: Deserialize<'de> + Serialize + Ord,
    V: Deserialize<'de> + Serialize,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MapVisitor<K, V>
        where
            K: Serialize,
            V: Serialize,
        {
            marker: PhantomData<fn() -> Map<K, V>>,
        }

        impl<K, V> MapVisitor<K, V>
        where
            K: Serialize,
            V: Serialize,
        {
            fn new() -> Self {
                MapVisitor {
                    marker: PhantomData,
                }
            }
        }

        impl<'de, K, V> Visitor<'de> for MapVisitor<K, V>
        where
            K: Deserialize<'de> + Serialize + Ord,
            V: Deserialize<'de> + Serialize,
        {
            type Value = Map<K, V>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("`array`")
            }

            fn visit_seq<S>(self, mut value: S) -> Result<Self::Value, S::Error>
            where
                S: SeqAccess<'de>,
            {
                match value.next_element::<String>()? {
                    Some(kind) => match kind.as_str() {
                        "map" => {
                            let values: Vec<(K, V)> = value.next_element()?.expect("map values");
                            let mut map: BTreeMap<K, V> = BTreeMap::new();
                            for (k, v) in values {
                                map.insert(k, v);
                            }
                            Ok(Map(map))
                        }
                        _ => Err(de::Error::invalid_value(de::Unexpected::Str(&kind), &"map")),
                    },
                    None => Err(de::Error::custom(
                        "`map` specified, but values not provided",
                    )),
                }
            }
        }

        deserializer.deserialize_seq(MapVisitor::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize() -> Result<(), serde_json::Error> {
        let expected = r#"["map",[["color","blue"]]]"#;
        let mut map: BTreeMap<String, String> = BTreeMap::new();
        map.insert("color".to_string(), "blue".to_string());
        let value = Map(map);
        let json = serde_json::to_string(&value)?;
        assert_eq!(json, expected);
        Ok(())
    }

    #[test]
    fn test_deserialize() -> Result<(), serde_json::Error> {
        let data = r#"["map",[["color","blue"]]]"#;
        let map: Map<String, String> = serde_json::from_str(data)?;
        assert_eq!(map.get("color").expect("color value"), "blue");
        Ok(())
    }
}
