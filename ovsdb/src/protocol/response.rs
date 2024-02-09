use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Serialize)]
pub struct Response {
    pub id: Option<super::Uuid>,
    pub result: Option<Value>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListResult<T> {
    pub rows: Vec<T>,
}
