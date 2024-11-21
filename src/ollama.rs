// TODO: move this mod into logics

use crate::protocol::ModelList;
use std::{env, panic};
use eyre::{eyre, Result};
use url::Url;

const DEFAULT_HOST: &str = "http://localhost:11434";

#[dynamic]
static HOST: Url = get_ollama_host().unwrap();

#[must_use]
pub fn path(path: &str) -> String {
    let mut host = HOST.clone();
    host.set_path(path);
    host.to_string()
}

#[must_use]
pub async fn get_models() -> Vec<String> {
    let uri = path("/api/tags");
    let mut models = reqwest::get(uri).await.unwrap()
        .json::<ModelList>().await.unwrap()
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

fn get_ollama_host() -> Result<Url> {
    let uri = env::var("OLLAMA_HOST").unwrap_or(DEFAULT_HOST.to_string());
    let mut uri = Url::parse(&uri)?;
    if uri.port().is_none() {
        if let Err(_) = uri.set_port(Some(11434)) {
            return Err(eyre!("error setting URI port"));
        };
    }
    Ok(uri)
}
