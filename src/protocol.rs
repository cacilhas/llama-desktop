use std::time::{SystemTime, UNIX_EPOCH};

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
pub struct AditionalParams {
    pub seed: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct Request {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
    pub options: AditionalParams,
    pub context: Option<Vec<u16>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Response {
    pub response: String,
    pub context: Option<Vec<u16>>,
    pub done: bool,
}

impl Default for AditionalParams {
    fn default() -> Self {
        Self { seed: get_seed() }
    }
}

fn get_seed() -> u32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|since| (since.as_millis() % u32::max_value() as u128) as u32)
        .unwrap_or(0)
}
