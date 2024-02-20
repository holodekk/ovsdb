use serde::{
    de::{self, Deserializer, MapAccess, Visitor},
    ser::SerializeMap,
    Deserialize, Serialize, Serializer,
};

use crate::protocol::method::{EchoParams, GetSchemaParams, TransactParams};

use super::{
    method::{Method, Params},
    Uuid,
};

/// Wire-format representation of an OVSDB method call.
#[derive(Debug)]
pub struct Request {
    id: Option<super::Uuid>,
    method: Method,
    params: Option<Box<dyn Params>>,
}

impl Request {
    /// Creates a new OVSDB request.
    #[must_use]
    pub fn new(method: Method, params: Option<Box<dyn Params>>) -> Self {
        Self {
            id: Some(super::Uuid::from(uuid::Uuid::new_v4())),
            method,
            params,
        }
    }

    /// Free-form id used for matching requests to responses
    #[must_use]
    pub fn id(&self) -> Option<&super::Uuid> {
        self.id.as_ref()
    }

    /// Actual OVSDB method being executed
    #[must_use]
    pub fn method(&self) -> Method {
        self.method
    }

    /// Parameters associated with this method call
    #[must_use]
    pub fn params(&self) -> Option<&dyn Params> {
        self.params.as_deref()
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
        match &self.params {
            Some(p) => map.serialize_entry("params", p)?,
            None => map.serialize_entry("params", &Vec::<i32>::new())?,
        }
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

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
                            let m = Method::try_from(k).map_err(de::Error::custom)?;
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
                        let params: Option<Box<dyn Params>> = match m {
                            Method::Echo => {
                                let v = params.ok_or("params").map_err(de::Error::missing_field)?;
                                let p: EchoParams =
                                    serde_json::from_value(v).map_err(de::Error::custom)?;
                                Some(Box::new(p))
                            }
                            Method::ListDatabases => None,
                            Method::GetSchema => {
                                let v = params.ok_or("params").map_err(de::Error::missing_field)?;
                                let p: GetSchemaParams =
                                    serde_json::from_value(v).map_err(de::Error::custom)?;
                                Some(Box::new(p))
                            }
                            Method::Transact => {
                                let v = params.ok_or("params").map_err(de::Error::missing_field)?;
                                let p: TransactParams =
                                    serde_json::from_value(v).map_err(de::Error::custom)?;
                                Some(Box::new(p))
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
