use std::{borrow::Borrow, thread, time::Duration};

use eframe::egui::text::LayoutJob;
use eframe::Frame;
use eframe::*;
use egui::*;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use egui_extras::install_image_loaders;
use reqwest::header;
use tokio::runtime::Runtime;
use tokio::sync::oneshot;

use crate::ollama;
use crate::protocol::{Request, Response};

#[derive(Debug)]
pub struct LlamaApp;

#[derive(Debug)]
struct State {
    logo: ImageSource<'static>,
    title_font: FontId,
    models: Vec<String>,
    selected_model: usize,
    input: String,
    output: String,
    retreiving: bool,
    context: Vec<i32>,
}

#[dynamic]
static mut STATE: State = State {
    logo: include_image!("assets/logo.png"),
    title_font: FontId::new(32.0, FontFamily::Name("arial".into())),
    models: Vec::new(),
    selected_model: 0,
    input: "Why the sky is blue?".to_owned(),
    output: String::new(),
    retreiving: false,
    context: Vec::new(),
};

#[dynamic]
static RUNTIME: Runtime = Runtime::new().unwrap();

#[dynamic]
static mut MD_CACHE: CommonMarkCache = CommonMarkCache::default();

/// LlamaApp is just a proxy for a module
impl LlamaApp {
    pub fn new(cc: &CreationContext<'_>, fonts: FontDefinitions) -> Self {
        install_image_loaders(&cc.egui_ctx);
        cc.egui_ctx.set_fonts(fonts);

        let (inp, mut out) = oneshot::channel::<Vec<String>>();
        RUNTIME.spawn(async move {
            inp.send(ollama::get_models().await).unwrap();
        });
        STATE.write().models = loop {
            match out.try_recv() {
                Ok(cache) => break cache,
                Err(oneshot::error::TryRecvError::Empty) => {
                    thread::sleep(Duration::from_millis(100));
                }
                Err(err) => {
                    eprintln!("failed to retrieve models: {}", err);
                    panic!("failed retrieving models");
                }
            }
        };

        Self
    }
}

impl App for LlamaApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        ctx.set_visuals(Visuals::dark());

        TopBottomPanel::top("title-panel")
            .exact_height(48.0)
            .show(ctx, |ui| {
                ui.columns(3, |uis| {
                    uis[0].with_layout(Layout::left_to_right(Align::Min), |ui| {
                        ui.add(
                            Image::new(STATE.read().logo.clone())
                                .fit_to_exact_size(Vec2 { x: 64.0, y: 64.0 }),
                        );
                    });

                    uis[1].with_layout(Layout::left_to_right(Align::Center), |ui| {
                        ui.label(
                            RichText::new("Llama Desktop")
                                .font(STATE.read().title_font.clone())
                                .strong(),
                        );
                    });

                    uis[2].with_layout(Layout::right_to_left(Align::Max), |ui| {
                        let mut state = STATE.write();
                        ComboBox::from_id_source(Id::new("models"))
                            .selected_text(&state.models[state.selected_model])
                            .show_ui(ui, |ui| {
                                let mut selected = state.selected_model;
                                for (idx, opt) in state.models.iter().enumerate() {
                                    let value =
                                        ui.selectable_value(&mut selected, idx, opt.clone());
                                    if value.clicked() {
                                        selected = idx;
                                    }
                                }
                                state.selected_model = selected;
                            });
                        ui.label(
                            RichText::new("Models:")
                                .font(state.title_font.clone())
                                .color(Color32::from_rgb(0x54, 0x10, 0x21))
                                .strong(),
                        );
                    });
                });
            });

        CentralPanel::default().show(ctx, |ui| {
            let size = ui.available_size();
            let text_size = Vec2::new(size.x, size.y / 3.0);
            ui.add_sized(
                text_size,
                TextEdit::multiline(&mut STATE.write().input)
                    .font(FontId::new(20.0, FontFamily::Proportional)),
            );

            ui.vertical_centered_justified(|ui| {
                let bt_text = LayoutJob::simple_singleline(
                    "Send".to_owned(),
                    FontId::new(20.0, FontFamily::Proportional),
                    Color32::from_rgb(0xff, 0xff, 0xff),
                );
                let send_button = Button::new(bt_text)
                    .rounding(10.0)
                    .shortcut_text("Ctrl+Enter");
                if STATE.read().retreiving {
                    ui.add_enabled(false, send_button);
                } else {
                    if send_button.ui(ui).clicked() {
                        STATE.write().retreiving = true;
                        RUNTIME.spawn(send());
                    }
                }
            });

            CommonMarkViewer::new("output").show_scrollable(
                ui,
                &mut MD_CACHE.write(),
                &STATE.read().output,
            );
        });
    }
}

async fn send() {
    let context = STATE.read().context.clone();
    let context: Option<Vec<u16>> = if context.is_empty() {
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
        .build()
        .unwrap();
    let payload = {
        let state = STATE.read();
        Request {
            model: state.models[state.selected_model].to_owned(),
            prompt: state.input.to_owned(),
            stream: true,
            context,
        }
    };
    let payload = serde_json::to_string(&payload).unwrap();
    let uri = ollama::path("/api/generate");
    let mut response = client.post(uri).body(payload).send().await.unwrap();
    if !response.status().is_success() {
        STATE.write().output = response.text().await.unwrap_or_else(|e| e.to_string());
        return;
    }

    STATE.write().output.clear();
    'read: while let Some(current) = response.chunk().await.unwrap() {
        let chunk: Response =
            serde_json::from_str(std::str::from_utf8(current.borrow()).unwrap()).unwrap();
        {
            let mut state = STATE.write();
            state.output += &chunk.response;
            if let Some(context) = chunk.context {
                state.context = context.iter().map(|e| *e as i32).collect::<Vec<i32>>();
            }
            if chunk.done {
                break 'read;
            }
        }
    }
    STATE.write().retreiving = false;
}
