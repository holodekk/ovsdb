use std::collections::BTreeMap;
use std::convert::From;
use std::fmt;
use std::ops::Deref;

use serde::{
    de::{Deserializer, MapAccess, Visitor},
    ser::{SerializeMap, Serializer},
    Deserialize, Serialize,
};

use super::Scalar;

/// Represents a `map` within ovsdb.
#[derive(Debug, PartialEq)]
pub struct Map(pub BTreeMap<String, Scalar>);

impl fmt::Display for Map {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{")?;
        for (idx, (key, value)) in self.0.iter().enumerate() {
            if idx > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}: {}", key, value)?;
        }
        write!(f, "}}")
    }
}

impl<K, V> From<BTreeMap<K, V>> for Map
where
    K: ToString + Ord,
    V: Into<Scalar>,
{
    fn from(value: BTreeMap<K, V>) -> Self {
        let mut map: BTreeMap<String, Scalar> = BTreeMap::new();
        for (k, v) in value {
            map.insert(k.to_string(), v.into());
        }
        Self(map)
    }
}

impl Deref for Map {
    type Target = BTreeMap<String, Scalar>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Serialize for Map {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.len()))?;
        for (k, v) in &self.0 {
            map.serialize_entry(&k, &v)?;
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for Map {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MapVisitor;

        impl<'de> Visitor<'de> for MapVisitor {
            type Value = Map;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("`map`")
            }

            fn visit_map<M>(self, mut value: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut map: BTreeMap<String, Scalar> = BTreeMap::new();

                while let Some((k, v)) = value.next_entry()? {
                    let key: String = k;
                    let value: Scalar = v;

                    map.insert(key, value);
                }

                Ok(Map(map))
            }
        }

        deserializer.deserialize_map(MapVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize() -> Result<(), serde_json::Error> {
        let expected = r#"{"color":"blue"}"#;
        let mut map: BTreeMap<String, Scalar> = BTreeMap::new();
        map.insert("color".to_string(), Scalar::from("blue"));
        let value = Map(map);
        let json = serde_json::to_string(&value)?;
        assert_eq!(json, expected);
        Ok(())
    }

    #[test]
    fn deserialize() -> Result<(), serde_json::Error> {
        let data = r#"{"color":"blue"}"#;
        let map: Map = serde_json::from_str(data)?;
        assert_eq!(map.get("color").unwrap(), &Scalar::from("blue"));
        Ok(())
    }
}
