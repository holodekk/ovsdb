use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Response<T> {
    pub result: T,
    pub error: Option<String>,
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct ListResult<T> {
    pub rows: Vec<T>,
}
