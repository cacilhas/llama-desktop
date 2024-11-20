use std::{fs::File, path::PathBuf};

use super::{set_model, STATE };
use chrono::Local;
use comrak::{markdown_to_html, Options};
use eyre::{eyre, Result};
use rfd::FileDialog;

#[derive(Clone, Copy, Debug)]
enum Step {
    ReadingHeader,
    ReadingQuestion,
}

use Step::*;

#[derive(Debug, Default)]
struct Parser(String, Vec<u32>);

pub async fn save_content(content: impl ToString) {
    let content = content.to_string();
    let cwd = STATE.read().cwd.to_owned();
    if let Some(path) = FileDialog::new()
        .set_title("Llama Desktop Save Context")
        .set_directory(cwd)
        .add_filter("Context", &["ctx"])
        .add_filter("HTML (not reloadable)", &["html", "htm"])
        .set_file_name(Local::now().format("%Y-%m-%d-%H%M.ctx").to_string())
        .save_file()
    {
        if let Err(err) = {
            if path
                .extension()
                .and_then(|e| e.to_str())
                .filter(|&e| e == "ctx")
                .is_some()
            {
                if let Some(parent) = path.parent().and_then(|e| e.to_str()) {
                    STATE.write().cwd = parent.to_owned();
                }
                STATE.write().reload = true;
                save_context(&content, path.as_path().to_str().unwrap()).await
            } else {
                if let Some(parent) = path.parent().and_then(|e| e.to_str()) {
                    STATE.write().cwd = parent.to_owned();
                }
                STATE.write().reload = true;
                save_html(&content, path.as_path().to_str().unwrap()).await
            }
        } {
            eprintln!("error saving to {:?}", &path);
            eprintln!("{:?}", err);
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
        .pick_file()
    {
        warn!("opening file: {:?}", &path);
        if let Some(parent) = path.parent().and_then(|e| e.to_str()) {
            STATE.write().cwd = parent.to_owned();
        }
        let mut parser = Parser(get_content(path.clone())?, Vec::new());
        parser.load().await?;
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
                    if let Some(model) = line.strip_prefix("model: ") {
                        if !set_model(model) {
                            warn!("using current model");
                        }
                    } else if let Some(context) = line.strip_prefix("context: ") {
                        self.1 = context
                            .split(",")
                            .map(|e| e.parse::<u32>().unwrap())
                            .collect::<Vec<_>>();
                    } else if line == "-----" {
                        warn!("end of headers");
                        step = ReadingQuestion;
                    }
                }

                _ => {
                    STATE.write().output.push_str(line);
                    STATE.write().output.push('\n');
                }
            }
        }

        warn!("context loaded");
        Ok(())
    }
}

async fn save_html(content: &str, path: &str) -> Result<()> {
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
    file.write_all(markdown_to_html(content, &Options::default()).as_bytes())?;
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
