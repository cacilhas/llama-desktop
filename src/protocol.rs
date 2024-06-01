use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct AIModel {
    pub name: String,
    pub modified_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelList {
    pub models: Vec<AIModel>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Request {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
    pub context: Option<Vec<u16>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Response {
    pub response: String,
    pub context: Option<Vec<u16>>,
    pub done: bool,
}
