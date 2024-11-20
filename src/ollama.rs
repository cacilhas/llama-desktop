// TODO: move this mod into logics

use crate::protocol::ModelList;
use std::{env, panic, process::exit};
use url::Url;

const DEFAULT_HOST: &str = "http://localhost:11434";

#[dynamic]
static HOST: String = get_ollama_host();

#[must_use]
pub fn path(path: &str) -> String {
    format!("{}{}", HOST.clone(), path)
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

fn get_ollama_host() -> String {
    let uri = env::var("OLLAMA_HOST").unwrap_or(DEFAULT_HOST.to_string());
    let uri = match Url::parse(&uri) {
        Ok(uri) => uri,
        Err(err) => {
            eprintln!(
                "error parsing \x1b[33m{}\x1b[0m: \x1b[31;1m{:?}\x1b[0m",
                uri, err,
            );
            eprintln!("fallback to \x1b[33;1m{}\x1b[0m", DEFAULT_HOST);
            eprintln!(
                "please review the content of environment variable \x1b[32mOLLAMA_HOST\x1b[0m",
            );
            exit(1);
        }
    };
    let Some(host) = uri.host_str() else {
        eprintln!("fail to parse {}", uri);
        exit(1);
    };
    format!(
        "{}://{}:{}",
        uri.scheme(),
        host,
        uri.port().unwrap_or(11434),
    )
}
