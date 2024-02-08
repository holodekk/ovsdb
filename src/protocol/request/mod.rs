use serde::{ser::SerializeMap, Serialize, Serializer};

mod method;
pub use method::*;
mod params;
pub use params::*;

#[derive(Debug)]
pub struct Request {
    pub id: uuid::Uuid,
    pub method: Method,
    pub params: Params,
}

impl Request {
    pub fn new(method: Method, params: Params) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
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
