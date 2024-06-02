use crate::protocol::ModelList;
use std::{env, fmt::Debug, panic, process};
use url::Url;

const DEFAULT_HOST: &'static str = "http://localhost:11434";

#[dynamic]
static HOST: String = get_ollama_host();

#[must_use]
pub fn path(path: &str) -> String {
    format!("{}{}", HOST.clone(), path)
}

#[must_use]
pub async fn get_models() -> Vec<String> {
    let uri = path("/api/tags");
    let mut models = panic(panic(reqwest::get(uri).await).json::<ModelList>().await).models;
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
            eprintln!("error parsing {}: {:?}", uri, err);
            eprintln!("fallback to {}", DEFAULT_HOST);
            eprintln!("please review the content of environment variable OLLAMA_HOST");
            panic(Url::parse(DEFAULT_HOST))
        }
    };
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

fn panic<T, E>(value: Result<T, E>) -> T
where
    E: Debug,
{
    match value {
        Ok(value) => value,
        Err(err) => {
            eprintln!("\x1b[31;1m[PANIC] couldn't initialise:\x1b[0m {:?}", err);
            process::exit(1);
        }
    }
}
