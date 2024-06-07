use std::{fs::File, path::PathBuf};

use super::{set_model, STATE, TIMEOUTS};
use crate::{
    ollama,
    protocol::{Request, Response},
};
use chrono::Local;
use comrak::{markdown_to_html, Options};
use eyre::{OptionExt, Result};
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
    use std::io::Write;

    let now = Local::now();
    let content = content.into();
    if let Some(path) = FileDialog::new()
        .add_filter("Llama Desktop file", &["html", "ldml"])
        .set_file_name(now.format("%Y-%m-%d-%H%M.ldml").to_string())
        .save_file()
    {
        let model = {
            let state = STATE.read();
            state.models[state.selected_model].to_owned()
        };
        _eprintln!("caching data");
        _dbg!(&model);
        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n");
        html.push_str("<html>\n");
        html.push_str("  <head>\n");
        html.push_str("    <title>");
        html.push_str(&STATE.read().title);
        html.push_str("</title>\n");
        html.push_str("  </head>\n");
        html.push_str("  <body>\n");
        html.push_str(&markdown_to_html(&content, &Options::default()));
        html.push_str("  </body>\n");
        html.push_str("\n<!--\n");
        html.push_str("---\n");
        html.push_str("filetype: llama markup\n");
        html.push_str("model: ");
        html.push_str(&model);
        html.push_str("\n---\n");
        html.push_str(&content.replace("<", "&lt;").replace(">", "&gt;"));
        html.push_str("&lt;!-- END OF DATA --&gt;\n");
        html.push_str("\n-->\n");
        html.push_str("</html>\n");
        _eprintln!("done caching");

        match File::create(&path) {
            Ok(mut file) => {
                _eprintln!("saving to {:?}", &path);
                if let Err(err) = file.write_all(html.as_bytes()) {
                    eprintln!("error writing {:?}", &path);
                    eprintln!("{:?}", err);
                }
            }
            Err(err) => {
                eprintln!("error opening {:?} for writing", &path);
                eprintln!("{:?}", err);
            }
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
    let path = FileDialog::new()
        .add_filter("HTML", &["html", "ldml"])
        .pick_file()
        .ok_or_eyre("error opening file")?;
    _eprintln!("opening file: {:?}", &path);
    Parser(get_content(path)?, Vec::new()).load().await?;
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
        _eprintln!("FINISHED");
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
        STATE.write().retrieving = true;
        let mut step = ReadingHTML;
        let mut question = String::new();
        let content = self.0.clone();

        for line in content.lines() {
            _dbg!(step);
            _dbg!(line);
            if line == "-->" || line == "</html>" {
                _eprintln!("IT SHOULD NEVER HAPPEN");
                continue;
            }
            let line = line.replace("&gt;", ">").replace("&lt;", "<");

            match step {
                ReadingHTML => {
                    if line == "filetype: llama markup" {
                        _eprintln!("end of HTML");
                        step = ReadingHeader;
                        continue;
                    }
                }

                ReadingHeader => {
                    if line.starts_with("model: ") {
                        let model = &line[7..];
                        if !set_model(model) {
                            _eprintln!("using current model");
                        }
                    } else if line == "---" {
                        _eprintln!("end of headers");
                        step = ReadingQuestion;
                        continue;
                    }
                }

                ReadingQuestion => {
                    if line == "<!-- END OF DATA -->" {
                        _eprintln!("end of data during a question");
                        self.feed_server(&question).await?;
                        break;
                    }

                    STATE.write().output.push_str(&line);
                    STATE.write().output.push_str("\n");

                    if line.starts_with("> ") {
                        question.push_str(&line[2..]);
                    } else {
                        _eprintln!("end of question");
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
                        _eprintln!("end of data");
                        break;
                    }

                    STATE.write().output.push_str(&line);
                    STATE.write().output.push_str("\n");

                    if line.starts_with("> ") {
                        _eprintln!("new question");
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
            _eprintln!("EMPTY QUESTION");
            return Ok(());
        }

        _eprintln!("feeding question: {}", &question);
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
                context: if self.1.is_empty() {
                    None
                } else {
                    Some(self.1.clone())
                },
            }
        };
        let uri = ollama::path("/api/generate");
        let payload = serde_json::to_string(&payload)?;
        _dbg!(&client, &uri, &payload);
        let response = time::timeout(timeout, client.post(uri).body(payload).send()).await??;

        if response.status().is_success() {
            let value: Response = serde_json::from_str(&response.text().await?)?;
            self.1 = value.context.ok_or_eyre("context")?;
            _dbg!(&self.1);
        }

        Ok(())
    }
}
