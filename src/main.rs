slint::include_modules!();

use eyre::*;
use ollama_rs::Ollama;
use slint::{SharedString, VecModel};
use std::{env, rc::Rc};
use url::Url;

#[tokio::main]
async fn main() -> Result<()> {
    let ollama = get_ollama()?;
    let models = get_models(&ollama).await?;
    let current_model = select_current_model(&models)?;

    let models = Rc::new(VecModel::from(
        models
            .iter()
            .map(|model| SharedString::from(model))
            .collect::<Vec<_>>(),
    ));

    let ui = AppWindow::new()?;
    ui.set_ai_models(models.into());
    ui.set_current_model(current_model.into());
    Ok(ui.run()?)
}

/*----------------------------------------------------------------------------*/

fn get_ollama() -> Result<Ollama> {
    let (host, port) = get_ollama_host()?;
    Ok(Ollama::new(host, port))
}

fn get_ollama_host() -> Result<(String, u16)> {
    let uri = env::var("OLLAMA_HOST").unwrap_or("http://localhost:11434".to_owned());
    let uri = Url::parse(&uri)?;
    Ok((
        format!(
            "{}://{}",
            uri.scheme(),
            uri.host_str()
                .ok_or_eyre(format!("fail to parse {}", uri))?
        ),
        uri.port().unwrap_or(11434),
    ))
}

async fn get_models(ollama: &Ollama) -> Result<Vec<String>> {
    let models = ollama
        .list_local_models()
        .await?
        .iter()
        .map(|model| model.name.to_owned())
        .collect::<Vec<_>>();

    if models.is_empty() {
        Err(eyre!("no model found"))
    } else {
        Ok(models)
    }
}

fn select_current_model(models: &Vec<String>) -> Result<String> {
    let mut current_model: Option<String> = None;
    for model in models.clone().iter() {
        if current_model.is_none() && model == "llama2" {
            current_model = Some(model.to_owned());
        } else if model.contains("phind-codellama") {
            current_model = Some(model.to_owned());
            break;
        }
    }
    Ok(current_model.unwrap_or(
        models
            .clone()
            .get(0)
            .ok_or_eyre("no model found")?
            .to_owned(),
    ))
}
