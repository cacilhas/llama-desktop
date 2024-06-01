use std::{thread, time::Duration};

use eframe::Frame;
use eframe::*;
use egui::*;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use egui_extras::install_image_loaders;
use tokio::runtime::Runtime;
use tokio::sync::oneshot;

use crate::ollama;

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
}

#[dynamic]
static mut STATE: State = State {
    logo: include_image!("assets/logo.png"),
    title_font: FontId::new(32.0, FontFamily::Name("arial".into())),
    models: Vec::new(),
    selected_model: 0,
    input: "Why the sky is blue?".to_owned(),
    output: String::new(),
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
        let mut state = STATE.write();
        ctx.set_visuals(Visuals::dark());

        TopBottomPanel::top("title-panel")
            .exact_height(48.0)
            .show(ctx, |ui| {
                ui.columns(3, |uis| {
                    uis[0].with_layout(Layout::left_to_right(Align::Min), |ui| {
                        ui.add(
                            Image::new(state.logo.clone())
                                .fit_to_exact_size(Vec2 { x: 64.0, y: 64.0 }),
                        );
                    });

                    uis[1].with_layout(Layout::left_to_right(Align::Center), |ui| {
                        ui.label(
                            RichText::new("Llama Desktop")
                                .font(state.title_font.clone())
                                .strong(),
                        );
                    });

                    uis[2].with_layout(Layout::right_to_left(Align::Max), |ui| {
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
            ui.add_sized(text_size, TextEdit::multiline(&mut state.input));

            ui.vertical_centered_justified(|ui| {
                if ui.button("send").clicked() {
                    RUNTIME.spawn(send());
                }
            });

            CommonMarkViewer::new("output").show(ui, &mut MD_CACHE.write(), &state.output);
        });
    }
}

async fn send() {}
