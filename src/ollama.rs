use crate::ModelList;
use eyre::*;
use std::env;
use url::Url;

static mut HOST: Option<String> = None;

pub fn init() -> Result<()> {
    unsafe {
        HOST = Some(get_ollama_host()?);
    }
    Ok(())
}

pub fn path(path: &str) -> Result<String> {
    unsafe { Ok(format!("{}{}", get_host()?, path)) }
}

pub async fn get_models() -> Result<Vec<String>> {
    let uri = path("/api/tags")?;
    let mut models = reqwest::get(uri).await?.json::<ModelList>().await?.models;
    models.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));
    let models = models
        .iter()
        .map(|model| model.name.to_owned())
        .collect::<Vec<_>>();

    if models.is_empty() {
        Err(eyre!("no model found"))
    } else {
        dbg!("Models received");
        Ok(models)
    }
}

unsafe fn get_host() -> Result<String> {
    HOST.clone().ok_or_eyre("OLLAMA_HOST not loaded yet")
}

fn get_ollama_host() -> Result<String> {
    let uri = env::var("OLLAMA_HOST").unwrap_or("http://localhost:11434".to_owned());
    let uri = Url::parse(&uri)?;
    Ok(format!(
        "{}://{}:{}",
        uri.scheme(),
        uri.host_str()
            .ok_or_eyre(format!("fail to parse {}", uri))?,
        uri.port().unwrap_or(11434),
    ))
}
