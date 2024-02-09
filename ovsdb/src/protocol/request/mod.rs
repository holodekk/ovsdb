use serde::{
    de::{self, Deserializer, MapAccess, Visitor},
    ser::SerializeMap,
    Deserialize, Serialize, Serializer,
};

use super::Uuid;

mod method;
pub use method::*;
mod params;
pub use params::*;

#[derive(Debug)]
pub struct Request {
    pub id: Option<super::Uuid>,
    pub method: Method,
    pub params: Params,
}

impl Request {
    pub fn new(method: Method, params: Params) -> Self {
        Self {
            id: Some(super::Uuid::from(uuid::Uuid::new_v4())),
            method,
            params,
        }
    }
}

impl Serialize for Request {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(3))?;
        map.serialize_entry("id", &self.id)?;
        map.serialize_entry("method", &self.method)?;
        map.serialize_entry("params", &self.params)?;
        map.end()
    }
}

impl<'de> Deserialize<'de> for Request {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RequestVisitor;

        impl<'de> Visitor<'de> for RequestVisitor {
            type Value = Request;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("`map`")
            }

            fn visit_map<S>(self, mut map: S) -> Result<Self::Value, S::Error>
            where
                S: MapAccess<'de>,
            {
                let mut id: Option<Uuid> = None;
                let mut method: Option<Method> = None;
                let mut params: Option<serde_json::Value> = None;
                while let Some((k, v)) = map.next_entry::<String, serde_json::Value>()? {
                    match k.as_str() {
                        "id" => {
                            let u = ::uuid::Uuid::parse_str(&k).map_err(de::Error::custom)?;
                            id = Some(Uuid::from(u));
                        }
                        "method" => {
                            let m = super::Method::try_from(k).map_err(de::Error::custom)?;
                            method = Some(m);
                        }
                        "params" => params = Some(v),
                        _ => {
                            // return Err(de::Error::unknown_field(k));
                            return Err(de::Error::unknown_field(&k, &["id", "method", "params"]));
                        }
                    }
                }

                match method {
                    Some(m) => {
                        let params = match m {
                            Method::Echo => {
                                let v = params.ok_or("params").map_err(de::Error::missing_field)?;
                                Params::Echo(serde_json::from_value(v).map_err(de::Error::custom)?)
                            }
                            Method::ListDatabases => Params::ListDatabases,
                            Method::GetSchema => {
                                let v = params.ok_or("params").map_err(de::Error::missing_field)?;
                                Params::GetSchema(
                                    serde_json::from_value(v).map_err(de::Error::custom)?,
                                )
                            }
                            Method::Transact => {
                                let v = params.ok_or("params").map_err(de::Error::missing_field)?;
                                Params::Transact(
                                    serde_json::from_value(v).map_err(de::Error::custom)?,
                                )
                            }
                        };
                        Ok(Request {
                            id,
                            method: m,
                            params,
                        })
                    }
                    None => Err(de::Error::missing_field("method")),
                }
            }
        }

        deserializer.deserialize_map(RequestVisitor)
    }
}
