use std::{process, thread, time::Duration};

use crate::fonts::set_font_size;
use crate::logics::*;
use crate::ollama;
use eframe::Frame;
use eframe::*;
use egui::*;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use egui_extras::install_image_loaders;
use tokio::runtime::Runtime;
use tokio::sync::oneshot;

#[derive(Debug)]
pub struct LlamaApp;

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
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        ctx.set_visuals(Visuals::dark());

        set_font_size(ctx, 20.0);
        if STATE.read().selected_model > STATE.read().models.len() {
            if let Some(storage) = frame.storage() {
                let selected_model = storage
                    .get_string("selected-model")
                    .unwrap_or("0".to_string());
                STATE.write().selected_model = selected_model.parse().unwrap_or(0);
            }
        }

        TopBottomPanel::top("title-panel")
            .exact_height(48.0)
            .show(ctx, |ui| {
                ui.columns(2, |uis| {
                    uis[0].with_layout(Layout::left_to_right(Align::Min), |ui| {
                        ui.add(
                            Image::new(STATE.read().logo.clone())
                                .fit_to_exact_size(Vec2 { x: 64.0, y: 64.0 }),
                        );

                        ui.label(
                            RichText::new("Llama Desktop")
                                .font(STATE.read().title_font.clone())
                                .strong(),
                        );
                    });

                    uis[1].with_layout(Layout::right_to_left(Align::Max), |ui| {
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
                                if selected != state.selected_model {
                                    state.selected_model = selected;
                                    if let Some(storage) = frame.storage_mut() {
                                        storage
                                            .set_string("selected-model", format!("{}", selected));
                                        storage.flush();
                                    }
                                }
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

        TopBottomPanel::bottom("footer")
            .exact_height(32.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Ctrl+Enter").strong());
                    ui.label("send");
                    ui.add_space(120.0);
                    ui.label(RichText::new("Ctrl+R").strong());
                    ui.label("reset");
                    ui.add_space(120.0);
                    ui.label(RichText::new("Ctrl+Q").strong());
                    ui.label("quit");
                });

                if ui.input(|st| st.modifiers.ctrl && st.key_pressed(Key::Q)) {
                    process::exit(0);
                }
            });

        CentralPanel::default().show(ctx, |ui| {
            let size = ui.available_size();
            let text_size = Vec2::new(size.x, size.y / 3.0);
            ui.add_sized(text_size, TextEdit::multiline(&mut STATE.write().input))
                .request_focus();

            ui.vertical_centered_justified(|ui| {
                let send_button = Button::new("Send")
                    .rounding(10.0)
                    .shortcut_text("Ctrl+Enter");
                if STATE.read().retreiving {
                    ui.add_enabled(false, send_button);
                } else {
                    if send_button.ui(ui).clicked()
                        || ui.input(|st| st.modifiers.ctrl && st.key_pressed(Key::Enter))
                    {
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

            if STATE.read().retreiving {
                Spinner::new().paint_at(
                    ui,
                    Rect::from_min_max(
                        Pos2::new(size.x / 2.0 - 16.0, size.y / 2.0 - 16.0),
                        Pos2::new(size.x / 2.0 + 16.0, size.y / 2.0 + 16.0),
                    ),
                );
            } else {
                if ui.input(|st| st.modifiers.ctrl && st.key_pressed(Key::R)) {
                    STATE.write().reset();
                }
            }
        });
    }
}

#[dynamic]
static RUNTIME: Runtime = Runtime::new().unwrap();

#[dynamic]
static mut MD_CACHE: CommonMarkCache = CommonMarkCache::default();
