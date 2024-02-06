use serde::{
    de::{self, Deserializer, MapAccess, SeqAccess, Visitor},
    ser::{SerializeSeq, Serializer},
    Deserialize, Serialize,
};

use super::{Map, Set, Uuid};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AtomKind {
    Map,
    Set,
    Uuid,
}

impl AtomKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Map => "map",
            Self::Set => "set",
            Self::Uuid => "uuid",
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Atom {
    Map(Map),
    Set(Set),
    Uuid(Uuid),
}

impl Serialize for Atom {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // self.0.serialize(serializer)
        // let map = Map::deserialize(de::value::MapAccessDeserializer::new(value))?;
        // Ok(Self::Value::Atom(Atom::Map(map)))
        let mut seq = serializer.serialize_seq(Some(2))?;
        match self {
            Atom::Map(m) => {
                seq.serialize_element("map")?;
                seq.serialize_element(&m)?;
            }
            Atom::Set(s) => {
                seq.serialize_element("set")?;
                seq.serialize_element(&s)?;
            }
            Atom::Uuid(u) => {
                seq.serialize_element("uuid")?;
                seq.serialize_element(&u)?;
            }
        }
        seq.end()
    }
}

impl<'de> Deserialize<'de> for Atom {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct AtomVisitor;

        impl<'de> Visitor<'de> for AtomVisitor {
            type Value = Atom;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("`array`")
            }

            fn visit_seq<S>(self, mut value: S) -> Result<Self::Value, S::Error>
            where
                S: SeqAccess<'de>,
            {
                let kind: AtomKind = value.next_element()?.unwrap();

                match kind {
                    AtomKind::Map => {
                        let map: Map = value.next_element()?.unwrap();
                        Ok(Self::Value::Map(map))
                    }
                    AtomKind::Set => {
                        let set: Set = value.next_element()?.unwrap();
                        Ok(Self::Value::Set(set))
                    }
                    AtomKind::Uuid => {
                        let uuid: Uuid = value.next_element()?.unwrap();
                        Ok(Self::Value::Uuid(uuid))
                    }
                }
            }

            fn visit_map<M>(self, value: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let map = Map::deserialize(de::value::MapAccessDeserializer::new(value))?;
                Ok(Self::Value::Map(map))
            }
        }
        deserializer.deserialize_any(AtomVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::Scalar;
    use crate::protocol::Value;

    #[test]
    fn serialize_map() -> Result<(), serde_json::Error> {
        let expected = r#"["map",{"color":"blue"}]"#;
        let map: Map = serde_json::from_str(r#"{"color":"blue"}"#)?;
        let value = Atom::Map(map);
        let json = serde_json::to_string(&value)?;
        assert_eq!(json, expected);
        Ok(())
    }

    #[test]
    fn serialize_set() -> Result<(), serde_json::Error> {
        let expected = r#"["set",["one","two"]]"#;
        let set: Set = serde_json::from_str(r#"["one","two"]"#)?;
        let value = Atom::Set(set);
        let json = serde_json::to_string(&value)?;
        assert_eq!(json, expected);
        Ok(())
    }

    #[test]
    fn serialize_uuid() -> Result<(), serde_json::Error> {
        let expected = r#"["uuid","36bef046-7da7-43a5-905a-c17899216fcb"]"#;
        let uuid: Uuid = serde_json::from_str(r#""36bef046-7da7-43a5-905a-c17899216fcb""#)?;
        let value = Atom::Uuid(uuid);
        let json = serde_json::to_string(&value)?;
        assert_eq!(json, expected);
        Ok(())
    }

    #[test]
    fn deserialize_map() -> Result<(), serde_json::Error> {
        let data = r#"["map",{"color":"blue"}]"#;
        let atom: Atom = serde_json::from_str(data)?;
        match atom {
            Atom::Map(m) => {
                assert_eq!(m.get("color").unwrap(), &Scalar::from("blue"));
            }
            _ => panic!("Invalid atom for map: {:#?}", atom),
        }
        Ok(())
    }

    #[test]
    fn deserialize_set() -> Result<(), serde_json::Error> {
        let data = r#"["set",["one","two","three"]]"#;
        let atom: Atom = serde_json::from_str(data)?;
        match atom {
            Atom::Set(s) => {
                assert_eq!(
                    s,
                    Set(vec![
                        Value::Scalar(Scalar::from("one")),
                        Value::Scalar(Scalar::from("two")),
                        Value::Scalar(Scalar::from("three"))
                    ])
                );
            }
            _ => panic!("Invalid atom for map: {:#?}", atom),
        }
        Ok(())
    }

    #[test]
    fn deserialize_uuid() -> Result<(), serde_json::Error> {
        let data = r#"["uuid","36bef046-7da7-43a5-905a-c17899216fcb"]"#;
        let atom: Atom = serde_json::from_str(data)?;
        match atom {
            Atom::Uuid(u) => assert_eq!(u.to_string(), "36bef046-7da7-43a5-905a-c17899216fcb"),
            _ => panic!("Invalid atom for uuid: {:#?}", atom),
        }
        Ok(())
    }
}
