use std::ops::Deref;

use serde::{
    de::{Deserializer, SeqAccess, Visitor},
    ser::{SerializeSeq, Serializer},
    Deserialize, Serialize,
};

use super::Value;

/// Represents a `set` within ovsdb.
#[derive(Debug, PartialEq)]
pub struct Set(pub Vec<Value>);

impl Deref for Set {
    type Target = Vec<Value>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Serialize for Set {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        for v in &self.0 {
            seq.serialize_element(&v)?;
        }
        seq.end()
    }
}

impl<'de> Deserialize<'de> for Set {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SetVisitor;

        impl<'de> Visitor<'de> for SetVisitor {
            type Value = Set;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("`array`")
            }

            fn visit_seq<S>(self, mut value: S) -> Result<Self::Value, S::Error>
            where
                S: SeqAccess<'de>,
            {
                let mut set: Vec<Value> = vec![];

                while let Some(v) = value.next_element()? {
                    let res: Value = v;
                    set.push(res);
                }

                Ok(Set(set))
            }
        }

        deserializer.deserialize_seq(SetVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    use crate::protocol::{Atom, Map, Scalar};

    #[test]
    fn serialize_scalars() -> Result<(), serde_json::Error> {
        let expected = r#"["one","two"]"#;
        let set = vec![
            Value::Scalar(Scalar::from("one")),
            Value::Scalar(Scalar::from("two")),
        ];
        let value = Set(set);
        let json = serde_json::to_string(&value)?;
        assert_eq!(json, expected);
        Ok(())
    }

    #[test]
    fn serialize_maps() -> Result<(), serde_json::Error> {
        let expected = r#"[["map",{"name":"one"}],["map",{"name":"two"}]]"#;
        let mut map1: BTreeMap<String, Scalar> = BTreeMap::new();
        map1.insert("name".to_string(), Scalar::from("one"));
        let mut map2: BTreeMap<String, Scalar> = BTreeMap::new();
        map2.insert("name".to_string(), Scalar::from("two"));
        let value = Set(vec![
            Value::Atom(Atom::Map(Map(map1))),
            Value::Atom(Atom::Map(Map(map2))),
        ]);
        let json = serde_json::to_string(&value)?;
        assert_eq!(json, expected);
        Ok(())
    }

    #[test]
    fn deserialize_scalars() -> Result<(), serde_json::Error> {
        let data = r#"["one","two"]"#;
        let set: Set = serde_json::from_str(data)?;
        assert_eq!(set.len(), 2);
        assert!(matches!(set.first().unwrap(), &Value::Scalar(_)));
        Ok(())
    }

    #[test]
    fn deserialize_maps() -> Result<(), serde_json::Error> {
        let data = r#"[{"name":"one"},{"name":"two"}]"#;
        let set: Set = serde_json::from_str(data)?;
        assert_eq!(set.len(), 2);
        assert!(matches!(set.first().unwrap(), &Value::Atom(Atom::Map(_))));
        Ok(())
    }
}
