use std::borrow::Borrow;
use std::error::Error;

use crate::helpers::{format_input_to_output, HR};
use crate::ollama;
use crate::protocol::{Request, Response};
use reqwest::header;
use tokio::time;

#[derive(Debug)]
pub struct State {
    pub models: Vec<String>,
    pub selected_model: usize,
    pub input: String,
    pub output: String,
    pub retrieving: bool,
    pub reload: bool,
    pub timeout_idx: usize,
    pub context: Vec<i32>,
}

impl State {
    pub fn reset(&mut self) {
        self.input = "Why the sky is blue?".to_owned();
        self.output = String::new();
        self.retrieving = false;
        self.reload = true;
        self.context = Vec::new();
    }
}

pub async fn send() {
    let context = STATE.read().context.clone();
    let context: Option<Vec<u16>> = if context.is_empty() {
        None
    } else {
        Some(context.iter().map(|e| *e as u16).collect())
    };
    let input = STATE.read().input.to_owned();
    STATE
        .write()
        .output
        .push_str(&format_input_to_output(input.clone()));
    STATE.write().output.push_str("\n\n");
    STATE.write().input.clear();

    let mut headers = header::HeaderMap::new();
    headers.insert(
        "Content-Type",
        header::HeaderValue::from_static("application/json"),
    );
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();
    let payload = {
        let state = STATE.read();
        Request {
            model: state.models[state.selected_model].to_owned(),
            prompt: input,
            stream: true,
            context,
        }
    };
    let payload = serde_json::to_string(&payload).unwrap();
    let uri = ollama::path("/api/generate");
    // TODO: make timeout configurable
    let timeout = time::Duration::from_secs(TIMEOUTS[STATE.read().timeout_idx] as u64);

    match time::timeout(timeout, client.post(uri).body(payload).send()).await {
        Ok(Ok(mut response)) => {
            if !response.status().is_success() {
                fail(response.text().await.unwrap_or_else(|e| e.to_string()));
                return;
            }

            'read: while let Ok(current) = time::timeout(timeout, response.chunk()).await {
                match current {
                    Ok(Some(current)) => {
                        let chunk: Response =
                            serde_json::from_str(std::str::from_utf8(current.borrow()).unwrap())
                                .unwrap();
                        let mut state = STATE.write();
                        state.output.push_str(&chunk.response);
                        if let Some(context) = chunk.context {
                            state.context = context.iter().map(|e| *e as i32).collect::<Vec<i32>>();
                        }
                        if chunk.done {
                            break 'read;
                        }
                    }

                    Ok(None) => {
                        fail("Ollama Server failed to respond");
                        return;
                    }

                    Err(err) => {
                        timeout_with_error(err);
                        return;
                    }
                }
            }
        }

        Ok(Err(err)) => {
            timeout_with_error(err);
            return;
        }

        Err(err) => {
            timeout_with_error(err);
            return;
        }
    }

    finish();
}

fn timeout_with_error(err: impl Error) {
    let mut res = String::new();
    res.push_str("Timeout waiting for Ollama Server:\n\n");
    res.push_str(&err.to_string());
    fail(res);
}

fn fail(message: impl Into<String> + Clone) {
    let res = format!("\n\n## ERROR\n{}\n", message.clone().into());
    STATE.write().output.push_str(&res);
    eprintln!("{}", message.into());
    finish();
}

fn finish() {
    let mut state = STATE.write();
    state.output.push_str(HR);
    state.retrieving = false;
    state.reload = true;
}

#[dynamic]
pub static mut STATE: State = State {
    models: Vec::new(),
    selected_model: usize::max_value(),
    input: "Why the sky is blue?".to_owned(),
    output: String::new(),
    retrieving: false,
    reload: true,
    timeout_idx: usize::max_value(),
    context: Vec::new(),
};

pub static TIMEOUTS: [usize; 7] = [10, 20, 30, 60, 120, 180, 300];
