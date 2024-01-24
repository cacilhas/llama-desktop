slint::include_modules!();

use eyre::*;
use slint::{spawn_local, SharedString, VecModel};
use std::{env, rc::Rc};
use url::Url;

static mut HOST: Option<String> = None;

#[tokio::main]
async fn main() -> Result<()> {
    unsafe {
        HOST = Some(get_ollama_host()?);
    }

    let models = get_models().await?;
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

    let ui_handle = ui.as_weak();
    ui.on_query(move |prompt| {
        let ui = ui_handle.unwrap();
        let prompt = prompt.to_string();
        let model = ui.get_current_model().to_string();

        dbg!("Querying", &model, &prompt);
        spawn_local(async move {
            dbg!("Spawning Ollama request");
            // use tokio_stream::StreamExt;
            // let mut stream = unsafe { OLLAMA.clone().unwrap() }
            //     .generate_stream(GenerationRequest::new(model, prompt))
            //     .await
            //     .unwrap();
            //
            // while let Some(res) = stream.next().await {
            //     for chunk in res.unwrap().iter() {
            //         ui.invoke_update_response(chunk.response.to_owned().into());
            //     }
            // }

            ui.invoke_response_done();
            dbg!("Response received");
        })
        .unwrap();
    });

    Ok(ui.run()?)
}

/*----------------------------------------------------------------------------*/

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

async fn get_models() -> Result<Vec<String>> {
    let uri = unsafe {
        format!(
            "{}/api/tags",
            HOST.clone().ok_or_eyre("OLLAMA_HOST not loaded yet")?
        )
    };
    let models = reqwest::get(uri)
        .await?
        .json::<ModelList>()
        .await?
        .models
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

fn select_current_model(models: &Vec<String>) -> Result<String> {
    let mut current_model: Option<String> = None;
    for model in models.clone().iter() {
        if current_model.is_none() && model == "llama2" {
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

#[derive(Debug, Clone, serde::Deserialize)]
struct Model {
    name: String,
    modified_at: String,
    size: u64,
    digest: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct ModelList {
    models: Vec<Model>,
}
