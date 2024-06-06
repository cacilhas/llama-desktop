use std::{fs::File, path::PathBuf};

use super::{set_model, STATE};
use crate::{
    helpers::HR,
    ollama,
    protocol::{Request, Response},
};
use chrono::Local;
use comrak::{markdown_to_html, Options};
use eyre::{eyre, OptionExt, Result};
use reqwest::header;
use rfd::FileDialog;

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
        .add_filter("HTML", &["html", "llama.html"])
        .set_file_name(now.format("%Y-%m-%d-%H%M.llama.html").to_string())
        .save_file()
    {
        let model = {
            let state = STATE.read();
            state.models[state.selected_model].to_owned()
        };
        let mut html = String::new();
        html.push_str(&markdown_to_html(&content, &Options::default()));
        html.push_str("\n<!--\n");
        html.push_str("---\n");
        html.push_str("model: ");
        html.push_str(&model);
        html.push_str("\n---\n");
        html.push_str(&content.replace("<", "&lt;").replace(">", "&gt;"));
        html.push_str("\n-->\n");

        match File::create(&path) {
            Ok(mut file) => {
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
        .add_filter("HTML", &["html", "llama.html"])
        .pick_file()
        .ok_or_eyre("error opening file")?;
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
        state.output.push_str(HR);
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
        let mut count_marks: usize = 0;
        let mut question = String::new();
        let content = self.0.clone();

        for line in content.lines() {
            let line = line.replace("&gt;", ">").replace("&lt;", "<");

            match step {
                ReadingHTML => {
                    if line == "</html>" {
                        step = ReadingHeader;
                        continue;
                    }
                }

                ReadingHeader => {
                    if line.starts_with("model: ") {
                        let model = &line[7..];
                        if !set_model(model) {
                            return Err(eyre![format!("unknown model name: {}", model)]);
                        }
                    } else if line == "---" {
                        count_marks += 1;
                    }
                    if count_marks == 2 {
                        step = ReadingQuestion;
                        continue;
                    }
                }

                ReadingQuestion => {
                    STATE.write().output.push_str(&line);
                    STATE.write().output.push_str("\n");

                    if line.starts_with("> ") {
                        question.push_str(&line[2..]);
                    } else {
                        step = ReadingAnswer;
                        self.feed_server(&question).await?;
                        question.clear();
                        continue;
                    }
                }

                ReadingAnswer => {
                    STATE.write().output.push_str(&line);
                    STATE.write().output.push_str("\n");

                    if line.starts_with("> ") {
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
            return Ok(());
        }

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
        let response = client.post(uri).body(payload).send().await?;

        if response.status().is_success() {
            let value: Response = serde_json::from_str(&response.text().await?)?;
            self.1 = value.context.ok_or_eyre("context")?;
        }

        Ok(())
    }
}
