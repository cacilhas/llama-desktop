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
use toml::Table;

#[derive(Debug)]
pub struct LlamaApp {
    pub logo: ImageSource<'static>,
    pub horizontal: ImageSource<'static>,
    pub vertical: ImageSource<'static>,
    pub title_font: FontId,
    pub small_font: FontId,
    pub horizontally: bool,
}

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

        Self {
            logo: include_image!("assets/logo.png"),
            horizontal: include_image!("assets/horizontal.png"),
            vertical: include_image!("assets/vertical.png"),
            title_font: FontId::new(32.0, FontFamily::Name("arial".into())),
            small_font: FontId::new(12.0, FontFamily::Name("arial".into())),
            horizontally: false,
        }
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

        TopBottomPanel::top("header")
            .exact_height(48.0)
            .show(ctx, |ui| {
                ui.columns(2, |uis| {
                    uis[0].with_layout(Layout::left_to_right(Align::Center), |ui| {
                        ui.add(
                            Image::new(self.logo.clone())
                                .fit_to_exact_size(Vec2 { x: 48.0, y: 48.0 }),
                        );

                        ui.label(
                            RichText::new("Llama Desktop")
                                .font(self.title_font.clone())
                                .strong(),
                        );

                        ui.label(
                            RichText::new(&format!("v{}", VERSION.to_string()))
                                .font(self.small_font.clone()),
                        );
                    });

                    uis[1].with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let mut state = STATE.write();
                        ComboBox::from_label(
                            RichText::new("Model:")
                                .font(self.title_font.clone())
                                .color(Color32::from_rgb(0x54, 0x10, 0x21))
                                .strong(),
                        )
                        .selected_text(&state.models[state.selected_model])
                        .show_ui(ui, |ui| {
                            let mut selected = state.selected_model;
                            for (idx, opt) in state.models.iter().enumerate() {
                                let value = ui.selectable_value(&mut selected, idx, opt.clone());
                                if value.clicked() {
                                    selected = idx;
                                }
                            }
                            if selected != state.selected_model {
                                state.selected_model = selected;
                                if let Some(storage) = frame.storage_mut() {
                                    storage.set_string("selected-model", format!("{}", selected));
                                    storage.flush();
                                }
                            }
                        });
                    });
                });
            });

        TopBottomPanel::bottom("footer")
            .exact_height(28.0)
            .show(ctx, |ui| {
                let mut sig_send = false;
                let mut sig_reset = false;
                let mut sig_quit = false;

                ui.columns(8, |uis| {
                    uis[0].with_layout(Layout::right_to_left(Align::Center), |ui| {
                        sig_send |= ui.label(RichText::new("Ctrl+Enter").strong()).clicked();
                    });
                    uis[1].with_layout(Layout::left_to_right(Align::Center), |ui| {
                        sig_send |= ui.label("send").clicked();
                    });
                    uis[2].with_layout(Layout::right_to_left(Align::Center), |ui| {
                        sig_reset |= ui.label(RichText::new("Ctrl+R").strong()).clicked();
                    });
                    uis[3].with_layout(Layout::left_to_right(Align::Center), |ui| {
                        sig_reset |= ui.label("reset").clicked();
                    });
                    uis[4].with_layout(Layout::right_to_left(Align::Center), |ui| {
                        sig_quit |= ui.label(RichText::new("Ctrl+Q").strong()).clicked();
                    });
                    uis[5].with_layout(Layout::left_to_right(Align::Center), |ui| {
                        sig_quit |= ui.label("quit").clicked();
                    });
                    uis[6].with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ImageButton::new(
                            Image::new(self.vertical.clone())
                                .fit_to_exact_size(Vec2 { x: 20.0, y: 20.0 }),
                        )
                        .ui(ui)
                        .clicked()
                        {
                            self.horizontally = false;
                            println!("{}", self.horizontally);
                        }
                    });
                    uis[7].with_layout(Layout::left_to_right(Align::Center), |ui| {
                        if ImageButton::new(
                            Image::new(self.horizontal.clone())
                                .fit_to_exact_size(Vec2 { x: 20.0, y: 20.0 }),
                        )
                        .ui(ui)
                        .clicked()
                        {
                            self.horizontally = true;
                            println!("{}", self.horizontally);
                        }
                    });
                });

                if !STATE.read().retreiving {
                    if sig_send || ui.input(|st| st.modifiers.ctrl && st.key_pressed(Key::Enter)) {
                        STATE.write().retreiving = true;
                        RUNTIME.spawn(send());
                    }
                    if sig_reset || ui.input(|st| st.modifiers.ctrl && st.key_pressed(Key::R)) {
                        STATE.write().reset();
                    }
                }

                if sig_quit || ui.input(|st| st.modifiers.ctrl && st.key_pressed(Key::Q)) {
                    process::exit(0);
                }
            });

        CentralPanel::default().show(ctx, |ui| {
            let size = ui.available_size();

            if self.horizontally {
                // Dispose text viewers horizontally
                let text_size = Vec2::new(size.x * 3.0 / 7.0, size.y);
                ui.horizontal_top(|ui| {
                    ScrollArea::vertical()
                        .max_width(text_size.x)
                        .max_height(text_size.y)
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            ui.add_sized(text_size, TextEdit::multiline(&mut STATE.write().input))
                                .request_focus();
                        });

                    CommonMarkViewer::new("output").show_scrollable(
                        ui,
                        &mut MD_CACHE.write(),
                        &STATE.read().output,
                    );
                });
            } else {
                // Dispose text viewers vertically (default)
                let text_size = Vec2::new(size.x, size.y / 3.0);
                ScrollArea::vertical()
                    .max_width(text_size.x)
                    .max_height(text_size.y)
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui.add_sized(text_size, TextEdit::multiline(&mut STATE.write().input))
                            .request_focus();
                    });

                CommonMarkViewer::new("output").show_scrollable(
                    ui,
                    &mut MD_CACHE.write(),
                    &STATE.read().output,
                );
            }

            if STATE.read().retreiving {
                Spinner::new().paint_at(
                    ui,
                    Rect::from_min_max(
                        Pos2::new(size.x / 2.0 - 16.0, size.y / 2.0 - 16.0),
                        Pos2::new(size.x / 2.0 + 16.0, size.y / 2.0 + 16.0),
                    ),
                );
            }
        });
    }
}

#[dynamic]
static RUNTIME: Runtime = Runtime::new().unwrap();

#[dynamic]
static mut MD_CACHE: CommonMarkCache = CommonMarkCache::default();

#[dynamic]
static VERSION: String = {
    let cargo = include_str!("../Cargo.toml").parse::<Table>().unwrap();
    let package = cargo["package"].as_table().unwrap();
    let version = package["version"].as_str().unwrap();
    return version.to_string();
};
