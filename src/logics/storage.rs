use std::{fs::File, io::Write};

use chrono::Local;
use comrak::{markdown_to_html, Options};
use rfd::FileDialog;

use super::STATE;

pub async fn save_content(content: impl Into<String>) {
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
                eprintln!("error opening {:?} for write", &path);
                eprintln!("{:?}", err);
            }
        }
    }
}
