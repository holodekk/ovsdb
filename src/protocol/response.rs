use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct Response {
    pub result: Value,
    pub error: Option<String>,
    pub id: uuid::Uuid,
}

#[derive(Debug, Deserialize)]
pub struct ListResult<T> {
    pub rows: Vec<T>,
}
