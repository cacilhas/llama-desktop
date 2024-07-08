use std::{fs::File, path::PathBuf};

use super::{set_model, STATE, TIMEOUTS};
use crate::{
    ollama,
    protocol::{AdditionalParams, Request, Response},
};
use chrono::Local;
use comrak::{markdown_to_html, Options};
use eyre::{eyre, OptionExt, Result};
use reqwest::header;
use rfd::FileDialog;
use tokio::time;

#[derive(Clone, Copy, Debug)]
enum Step {
    ReadingHTML,
    ReadingHeader,
    ReadingQuestion,
    ReadingAnswer,
}

use Step::*;

#[derive(Debug, Default)]
struct Parser(String, Vec<u16>);

pub async fn save_content(content: impl Into<String>) {
    let content = content.into();
    let cwd = STATE.read().cwd.to_owned();
    if let Some(path) = FileDialog::new()
        .set_title("Llama Desktop Save Context")
        .set_directory(cwd)
        .add_filter("Context", &["ctx"])
        .add_filter("HTML (not loadable)", &["html", "htm"])
        .set_file_name(Local::now().format("%Y-%m-%d-%H%M.ctx").to_string())
        .save_file()
    {
        match if path
            .extension()
            .map(|e| e.to_str())
            .flatten()
            .filter(|&e| e == "ctx")
            .is_some()
        {
            if let Some(parent) = path.parent().map(|e| e.to_str()).flatten() {
                STATE.write().cwd = parent.to_owned();
            }
            STATE.write().reload = true;
            save_context(&content, path.as_path().to_str().unwrap()).await
        } else {
            if let Some(parent) = path.parent().map(|e| e.to_str()).flatten() {
                STATE.write().cwd = parent.to_owned();
            }
            STATE.write().reload = true;
            save_html(&content, path.as_path().to_str().unwrap()).await
        } {
            Err(err) => {
                eprintln!("error saving to {:?}", &path);
                eprintln!("{:?}", err);
            }
            Ok(()) => (),
        }
    }
}

pub async fn load() {
    if let Err(err) = do_load().await {
        println!("error reading file");
        println!("{:?}", err);
    }
}

async fn do_load() -> Result<()> {
    let cwd = STATE.read().cwd.to_owned();
    if let Some(path) = FileDialog::new()
        .set_title("Llama Desktop Load Context")
        .set_directory(cwd)
        .add_filter("Context", &["ctx"])
        .add_filter("Legacy", &["ldml"])
        .pick_file()
    {
        warn!("opening file: {:?}", &path);
        if let Some(parent) = path.parent().map(|e| e.to_str()).flatten() {
            STATE.write().cwd = parent.to_owned();
        }
        let mut parser = Parser(get_content(path.clone())?, Vec::new());
        match path.extension().map(|s| s.to_str().unwrap()) {
            Some("ctx") => parser.load().await?,
            _ => parser.load_legacy().await?,
        }
    }
    Ok(())
}

fn get_content(path: PathBuf) -> Result<String> {
    use std::io::Read;

    let mut file = File::open(&path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

impl Drop for Parser {
    fn drop(&mut self) {
        warn!("finished");
        let mut state = STATE.write();
        state.retrieving = false;
        state.escape = false;
        state.reload = true;
        if !self.1.is_empty() {
            // This is the whole point
            state.context = self.1.clone();
        }
    }
}

impl Parser {
    async fn load(&mut self) -> Result<()> {
        warn!("loading context");
        STATE.write().retrieving = true;
        let content = self.0.to_owned();
        let mut step = ReadingHeader;
        for line in content.lines() {
            debug!(step, line);

            match step {
                ReadingHeader => {
                    if line.starts_with("model: ") {
                        let model = &line[7..];
                        if !set_model(model) {
                            warn!("using current model");
                        }
                    } else if line.starts_with("context: ") {
                        let context = &line[9..];
                        self.1 = context
                            .split(",")
                            .map(|e| e.parse::<u16>().unwrap())
                            .collect::<Vec<_>>();
                    } else if line == "-----" {
                        warn!("end of headers");
                        step = ReadingQuestion;
                    }
                }

                _ => {
                    STATE.write().output.push_str(&line);
                    STATE.write().output.push_str("\n");
                }
            }
        }

        warn!("context loaded");
        Ok(())
    }

    async fn load_legacy(&mut self) -> Result<()> {
        warn!("loading legacy file");
        STATE.write().retrieving = true;
        let mut step = ReadingHTML;
        let mut question = String::new();
        let content = self.0.to_owned();

        for line in content.lines() {
            debug!(step, line);
            if line == "-->" || line == "</html>" {
                warn!("IT SHOULD NEVER HAPPEN");
                continue;
            }
            let line = line.replace("&gt;", ">").replace("&lt;", "<");

            match step {
                ReadingHTML => {
                    if line == "filetype: llama markup" {
                        warn!("end of HTML");
                        step = ReadingHeader;
                        continue;
                    }
                }

                ReadingHeader => {
                    if line.starts_with("model: ") {
                        let model = &line[7..];
                        if !set_model(model) {
                            warn!("using current model");
                        }
                    } else if line == "---" {
                        warn!("end of headers");
                        step = ReadingQuestion;
                        continue;
                    }
                }

                ReadingQuestion => {
                    if line == "<!-- END OF DATA -->" {
                        warn!("end of data during a question");
                        self.feed_server(&question).await?;
                        break;
                    }

                    STATE.write().output.push_str(&line);
                    STATE.write().output.push_str("\n");

                    if line.starts_with("> ") {
                        question.push_str(&line[2..]);
                    } else {
                        warn!("end of question");
                        step = ReadingAnswer;
                        if let Err(err) = self.feed_server(&question).await {
                            eprintln!("{:?}", err);
                        }
                        question.clear();
                        continue;
                    }
                }

                ReadingAnswer => {
                    if line == "<!-- END OF DATA -->" {
                        warn!("end of data");
                        break;
                    }

                    STATE.write().output.push_str(&line);
                    STATE.write().output.push_str("\n");

                    if line.starts_with("> ") {
                        warn!("new question");
                        question.push_str(&line[2..]);
                        step = ReadingQuestion;
                        continue;
                    }
                }
            }
        }

        Ok(())
    }

    async fn feed_server(&mut self, question: impl Into<String>) -> Result<()> {
        let question = question.into();
        if question.is_empty() {
            warn!("EMPTY QUESTION");
            return Ok(());
        }

        warn!("feeding question: {}", &question);
        let timeout = time::Duration::from_secs(TIMEOUTS[STATE.read().timeout_idx] as u64);
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Content-Type",
            header::HeaderValue::from_static("application/json"),
        );
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;
        let payload = {
            Request {
                model: STATE.read().models[STATE.read().selected_model].to_owned(),
                prompt: question,
                stream: false,
                options: AdditionalParams::default(),
                context: if self.1.is_empty() {
                    None
                } else {
                    Some(self.1.clone())
                },
            }
        };
        let uri = ollama::path("/api/generate");
        let payload = serde_json::to_string(&payload)?;
        debug!(&client, &uri, &payload);
        let response = time::timeout(timeout, client.post(uri).body(payload).send()).await??;

        if response.status().is_success() {
            let value: Response = serde_json::from_str(&response.text().await?)?;
            self.1 = value.context.ok_or_eyre("context")?;
            debug!(&self.1);
        }

        Ok(())
    }
}

pub async fn save_html(content: &str, path: &str) -> Result<()> {
    use std::io::Write;

    warn!("saving to HTML: {}", path);
    let mut file = File::create(path)?;
    file.write_all(b"<!DOCTYPE html>\n")?;
    file.write_all(b"<html>\n")?;
    file.write_all(b"  <head>\n")?;
    file.write_all(b"    <title>")?;
    file.write_all(STATE.read().title.as_bytes())?;
    file.write_all(b"</title>\n")?;
    file.write_all(b"  </head>\n")?;
    file.write_all(b"  <body>\n")?;
    file.write_all(markdown_to_html(&content, &Options::default()).as_bytes())?;
    file.write_all(b"  </body>\n")?;
    file.write_all(b"</html>\n")?;
    warn!("saved to HTML");

    Ok(())
}

pub async fn save_context(content: &str, path: &str) -> Result<()> {
    use std::io::Write;

    warn!("saving context to {}", path);
    let model = {
        let state = STATE.read();
        state.models[state.selected_model].to_owned()
    };
    let context = STATE.read().context.to_owned();
    if context.is_empty() {
        return Err(eyre!("no context to save"));
    }
    let mut file = File::create(path)?;
    file.write_all(b"model: ")?;
    file.write_all(model.as_bytes())?;
    file.write_all(b"\ncontext: ")?;
    file.write_all(format!("{}", context[0]).as_bytes())?;
    for part in context.to_owned().iter().skip(1) {
        file.write_all(format!(",{}", part).as_bytes())?;
    }
    file.write_all(b"\n-----\n")?;
    file.write_all(content.as_bytes())?;
    warn!("context saved");

    Ok(())
}
