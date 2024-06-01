use crate::protocol::ModelList;
use std::{env, panic};
use url::Url;

static mut HOST: Option<String> = None;

pub fn init() {
    unsafe {
        HOST = Some(get_ollama_host());
    }
}

#[must_use]
pub fn path(path: &str) -> String {
    unsafe { format!("{}{}", get_host(), path) }
}

#[must_use]
pub async fn get_models() -> Vec<String> {
    let uri = path("/api/tags");
    let mut models = reqwest::get(uri)
        .await
        .unwrap()
        .json::<ModelList>()
        .await
        .unwrap()
        .models;
    models.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));
    let models = models
        .iter()
        .map(|model| model.name.to_owned())
        .collect::<Vec<_>>();

    if models.is_empty() {
        panic!("no model found");
    }
    models
}

unsafe fn get_host() -> String {
    match HOST.clone() {
        Some(host) => host,
        None => panic!("OLLAMA_HOST not loaded yet"),
    }
}

fn get_ollama_host() -> String {
    let uri = env::var("OLLAMA_HOST").unwrap_or("http://localhost:11434".to_owned());
    let uri = Url::parse(&uri).unwrap();
    let host = match uri.host_str() {
        Some(host) => host,
        None => {
            eprintln!("fail to parse {}", uri);
            panic!("parsing error");
        }
    };
    format!(
        "{}://{}:{}",
        uri.scheme(),
        host,
        uri.port().unwrap_or(11434),
    )
}
