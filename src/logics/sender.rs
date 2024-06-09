use std::borrow::Borrow;

use super::state::STATE;
use super::timeouts::TIMEOUTS;
use crate::helpers::{format_input_to_output, HR};
use crate::ollama;
use crate::protocol::{AditionalParams, Request, Response};
use eyre::{eyre, Result};
use reqwest::header;
use tokio::time;

#[derive(Debug)]
pub struct Sender(f32);

impl Drop for Sender {
    fn drop(&mut self) {
        warn!("FINISHED");
        let mut state = STATE.write();
        state.output.push_str(HR);
        state.retrieving = false;
        state.escape = false;
        state.reload = true;
    }
}

impl Sender {
    pub fn new(temperature: f32) -> Self {
        Self(temperature)
    }

    pub async fn send(self) {
        STATE.write().retrieving = true;

        if let Err(err) = self.do_send().await {
            let mut state = STATE.write();
            warn!("{:?}", err);
            state.output.push_str("\n## ERROR:\n");
            state.output.push_str(&format!("{}", err));
        }
    }

    async fn do_send(&self) -> Result<()> {
        warn!("SENDING CONTENT");

        let context = STATE.read().context.clone();
        debug!(&context);

        let input = STATE.read().input.to_owned();
        debug!(&input);

        if input.is_empty() {
            return Err(eyre!("empty question"));
        }

        if STATE.read().title.is_empty() {
            STATE.write().title = input.to_owned();
        }

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
        debug!(&headers);
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;
        let mut payload = {
            let state = STATE.read();
            Request {
                model: state.models[state.selected_model].to_owned(),
                prompt: input,
                stream: true,
                options: AditionalParams::default(),
                context: if context.is_empty() {
                    None
                } else {
                    Some(context)
                },
            }
        };
        payload.options.temperature = self.0;
        debug!(&payload);
        let payload = serde_json::to_string(&payload)?;
        let uri = ollama::path("/api/generate");
        debug!(&uri);
        let timeout = time::Duration::from_secs(TIMEOUTS[STATE.read().timeout_idx] as u64);
        debug!(&timeout);

        self.check_escape()?;
        let mut response = time::timeout(timeout, client.post(uri).body(payload).send()).await??;
        if !response.status().is_success() {
            return Err(eyre![response.text().await?]);
        }

        debug!(&response);
        self.check_escape()?;
        'read: while let Some(current) = time::timeout(timeout, response.chunk()).await?? {
            debug!(&current);
            self.check_escape()?;
            let chunk: Response = serde_json::from_str(std::str::from_utf8(current.borrow())?)?;
            let mut state = STATE.write();
            state.output.push_str(&chunk.response);
            if let Some(context) = chunk.context {
                state.context = context.clone();
            }
            if chunk.done {
                break 'read;
            }
        }

        warn!("DONE");
        Ok(())
    }

    fn check_escape(&self) -> Result<()> {
        if STATE.read().escape {
            Err(eyre::eyre!["Escape key pressed."])
        } else {
            Ok(())
        }
    }
}
