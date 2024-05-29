mod ollama;
mod protocol;

slint::include_modules!();

use crate::protocol::*;
use eyre::*;
use reqwest::header;
use slint::{spawn_local, Model, SharedString, VecModel};
use std::{borrow::Borrow, env, rc::Rc, str};

#[tokio::main]
async fn main() -> Result<()> {
    ollama::init()?;
    let models = ollama::get_models().await?;
    let current_model = select_current_model(&models)?;

    let models = Rc::new(VecModel::from(
        models.iter().map(SharedString::from).collect::<Vec<_>>(),
    ));

    let ui = AppWindow::new()?;
    ui.set_ai_models(models.into());
    ui.set_current_model(current_model.into());

    let ui_handle = ui.as_weak();
    ui.on_query(move |prompt| {
        let ui = ui_handle.unwrap();
        let prompt = prompt.to_string();
        spawn_local(async move {
            query(ui, prompt).await.unwrap();
        })
        .unwrap();
    });

    Ok(ui.run()?)
}

/*----------------------------------------------------------------------------*/

async fn query(ui: AppWindow, prompt: String) -> Result<()> {
    let model = ui.get_current_model().to_string();
    let context: Vec<i32> = ui.get_chat_context().iter().collect();
    let context = if context.is_empty() {
        None
    } else {
        Some(context.iter().map(|e| *e as u16).collect())
    };

    let mut headers = header::HeaderMap::new();
    headers.insert(
        "Content-Type",
        header::HeaderValue::from_static("application/json"),
    );
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;
    let payload = Request {
        model,
        prompt,
        stream: true,
        context,
    };
    let payload = serde_json::to_string(&payload)?;
    let uri = ollama::path("/api/generate")?;
    let mut response = client.post(uri).body(payload).send().await?;
    if !response.status().is_success() {
        let err = response.text().await.unwrap_or_else(|e| e.to_string());
        ui.invoke_update_response(err.to_owned().into());
        ui.invoke_response_done();
        return Ok(());
    }

    while let Some(current) = response.chunk().await? {
        let chunk: Response = serde_json::from_str(str::from_utf8(current.borrow())?)?;
        ui.invoke_update_response(chunk.response.to_owned().into());
        if let Some(context) = chunk.context {
            let context = Rc::new(VecModel::from(
                context.iter().map(|e| *e as i32).collect::<Vec<i32>>(),
            ));
            ui.set_chat_context(context.into());
        }
    }

    ui.invoke_response_done();

    Ok(())
}

fn select_current_model(models: &[String]) -> Result<String> {
    let mut current_model: Option<String> = None;
    for model in models.iter() {
        if current_model.is_none() && model == "llama2" {
            current_model = Some(model.to_owned());
            break;
        }
    }
    Ok(current_model.unwrap_or(models.first().ok_or_eyre("no model found")?.to_owned()))
}
