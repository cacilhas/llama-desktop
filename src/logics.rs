use std::borrow::Borrow;
use std::error::Error;

use crate::helpers::{format_input_to_output, HR};
use crate::ollama;
use crate::protocol::{Request, Response};
use eyre::Result;
use reqwest::header;
use tokio::time;
use toml::Table;

#[derive(Debug)]
pub struct State {
    pub models: Vec<String>,
    pub selected_model: usize,
    pub input: String,
    pub output: String,
    pub retrieving: bool,
    pub reload: bool,
    pub timeout_idx: usize,
    pub escape: bool,
    pub context: Vec<i32>,
}

impl State {
    pub fn reset(&mut self) {
        _eprintln!("RESETING STATE");
        self.input = "Why the sky is blue?".to_owned();
        self.output = String::new();
        self.retrieving = false;
        self.reload = true;
        self.context = Vec::new();
        _dbg!(self);
    }
}

#[derive(Debug, Default)]
pub struct Sender;

impl Drop for Sender {
    fn drop(&mut self) {
        _eprintln!("FINISHED");
        let mut state = STATE.write();
        state.output.push_str(HR);
        state.retrieving = false;
        state.escape = false;
        state.reload = true;
    }
}

impl Sender {
    pub async fn send(self) {
        if let Err(err) = self.do_send().await {
            let mut state = STATE.write();
            state.output.push_str("\n## ERROR:\n");
            state.output.push_str(&format!("{}", err));
        }
    }

    async fn do_send(&self) -> Result<()> {
        _eprintln!("SENDING CONTENT");

        let context = STATE.read().context.clone();
        let context: Option<Vec<u16>> = if context.is_empty() {
            None
        } else {
            Some(context.iter().map(|e| *e as u16).collect())
        };
        _dbg!(&context);

        let input = STATE.read().input.to_owned();
        _dbg!(&input);
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
        _dbg!(&headers);
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;
        let payload = {
            let state = STATE.read();
            Request {
                model: state.models[state.selected_model].to_owned(),
                prompt: input,
                stream: true,
                context,
            }
        };
        _dbg!(&payload);
        let payload = serde_json::to_string(&payload)?;
        let uri = ollama::path("/api/generate");
        _dbg!(&uri);
        let timeout = time::Duration::from_secs(TIMEOUTS[STATE.read().timeout_idx] as u64);
        _dbg!(&timeout);

        self.check_escape()?;
        match time::timeout(timeout, client.post(uri).body(payload).send()).await {
            Ok(Ok(mut response)) => {
                if !response.status().is_success() {
                    return self.fail(response.text().await.unwrap_or_else(|e| e.to_string()));
                }

                _dbg!(&response);
                self.check_escape()?;
                'read: while let Ok(current) = time::timeout(timeout, response.chunk()).await {
                    match current {
                        Ok(Some(current)) => {
                            _dbg!(&current);
                            self.check_escape()?;
                            let chunk: Response =
                                serde_json::from_str(std::str::from_utf8(current.borrow())?)?;
                            let mut state = STATE.write();
                            state.output.push_str(&chunk.response);
                            if let Some(context) = chunk.context {
                                state.context =
                                    context.iter().map(|e| *e as i32).collect::<Vec<i32>>();
                            }
                            if chunk.done {
                                _eprintln!("DONE");
                                break 'read;
                            }
                        }

                        Ok(None) => {
                            return self.fail("Ollama Server failed to respond");
                        }

                        Err(err) => {
                            return self.timeout_with_error(err);
                        }
                    }
                }
            }

            Ok(Err(err)) => {
                return self.timeout_with_error(err);
            }

            Err(err) => {
                return self.timeout_with_error(err);
            }
        }

        Ok(())
    }

    fn timeout_with_error(&self, err: impl Error) -> Result<()> {
        let mut res = String::new();
        res.push_str("Timeout waiting for Ollama Server:\n\n");
        res.push_str(&err.to_string());
        self.fail(res)
    }

    fn fail(&self, message: impl Into<String> + Clone) -> Result<()> {
        _eprintln!("{}", message.clone().into());
        Err(eyre::eyre![message.into()])
    }

    fn check_escape(&self) -> Result<()> {
        if STATE.read().escape {
            Err(eyre::eyre!["Escape key pressed."])
        } else {
            Ok(())
        }
    }
}

#[dynamic]
pub static mut STATE: State = State {
    models: Vec::new(),
    selected_model: usize::max_value(),
    input: "Why the sky is blue?".to_owned(),
    output: String::new(),
    retrieving: false,
    reload: true,
    escape: false,
    timeout_idx: usize::max_value(),
    context: Vec::new(),
};

pub static TIMEOUTS: [usize; 7] = [10, 20, 30, 60, 120, 180, 300];

#[dynamic]
pub static VERSION: String = {
    let cargo = include_str!("../Cargo.toml").parse::<Table>().unwrap();
    let package = cargo["package"].as_table().unwrap();
    let version = package["version"].as_str().unwrap();
    return version.to_string();
};
