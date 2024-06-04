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
    logo: ImageSource<'static>,
    horizontal: ImageSource<'static>,
    vertical: ImageSource<'static>,
    title_font: FontId,
    small_font: FontId,
    box_layout: BoxLayout,
}

#[derive(Debug, Default, Eq, PartialEq)]
enum BoxLayout {
    Horizontally,
    Vertically,
    #[default]
    NotSet,
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
            box_layout: BoxLayout::default(),
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
        if self.box_layout == BoxLayout::NotSet {
            if let Some(storage) = frame.storage() {
                if storage.get_string("layout").unwrap_or("V".to_string()) == "H".to_string() {
                    self.box_layout = BoxLayout::Horizontally;
                } else {
                    self.box_layout = BoxLayout::Vertically;
                }
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
                let retrieving = STATE.read().retrieving;

                ui.columns(8, |uis| {
                    uis[0].with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let text = if retrieving {
                            RichText::new("Ctrl+Enter").weak()
                        } else {
                            RichText::new("Ctrl+Enter").strong()
                        };
                        sig_send |= ui.label(text).clicked();
                    });
                    uis[1].with_layout(Layout::left_to_right(Align::Center), |ui| {
                        let text = if retrieving {
                            RichText::new("send").weak()
                        } else {
                            RichText::new("send")
                        };
                        sig_send |= ui.label(text).clicked();
                    });
                    uis[2].with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let text = if retrieving {
                            RichText::new("Ctrl+R").weak()
                        } else {
                            RichText::new("Ctrl+R").strong()
                        };
                        sig_reset |= ui.label(text).clicked();
                    });
                    uis[3].with_layout(Layout::left_to_right(Align::Center), |ui| {
                        let text = if retrieving {
                            RichText::new("reset").weak()
                        } else {
                            RichText::new("reset")
                        };
                        sig_send |= ui.label(text).clicked();
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
                            self.box_layout = BoxLayout::Vertically;
                            if let Some(storage) = frame.storage_mut() {
                                storage.set_string("layout", "V".to_string());
                                storage.flush();
                            }
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
                            self.box_layout = BoxLayout::Horizontally;
                            if let Some(storage) = frame.storage_mut() {
                                storage.set_string("layout", "H".to_string());
                                storage.flush();
                            }
                        }
                    });
                });

                if !retrieving {
                    if sig_send || ui.input(|st| st.modifiers.ctrl && st.key_pressed(Key::Enter)) {
                        STATE.write().retrieving = true;
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
            let mut body: Option<Rect> = None;

            match self.box_layout {
                BoxLayout::Horizontally => {
                    // Dispose text viewers horizontally
                    let text_size = Vec2::new(size.x * 3.0 / 7.0, size.y);
                    ui.horizontal_top(|ui| {
                        ScrollArea::vertical()
                            .max_width(text_size.x)
                            .max_height(text_size.y)
                            .auto_shrink([false; 2])
                            .show(ui, |ui| {
                                let input = ui.add_sized(
                                    text_size,
                                    TextEdit::multiline(&mut STATE.write().input),
                                );
                                if STATE.read().reload {
                                    input.request_focus();
                                }
                            });

                        body = Some(ui.available_rect_before_wrap());
                        CommonMarkViewer::new("output").show_scrollable(
                            ui,
                            &mut MD_CACHE.write(),
                            &STATE.read().output,
                        );
                    });
                }
                BoxLayout::Vertically => {
                    // Dispose text viewers vertically (default)
                    let text_size = Vec2::new(size.x, size.y / 3.0);
                    ScrollArea::vertical()
                        .max_width(text_size.x)
                        .max_height(text_size.y)
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            let input = ui.add_sized(
                                text_size,
                                TextEdit::multiline(&mut STATE.write().input),
                            );
                            if STATE.read().reload {
                                input.request_focus();
                            }
                        });

                    body = Some(ui.available_rect_before_wrap());
                    CommonMarkViewer::new("output").show_scrollable(
                        ui,
                        &mut MD_CACHE.write(),
                        &STATE.read().output,
                    );
                }
                BoxLayout::NotSet => (),
            }

            if STATE.read().retrieving {
                let radius: f32 = 16.0;
                let (min, max) = match body {
                    Some(rect) => {
                        let half_width = rect.width() / 2.0;
                        let half_height = rect.height() / 2.0;
                        let x = rect.min.x;
                        let y = rect.min.y;
                        (
                            Pos2::new(half_width - radius + x, half_height - radius + y),
                            Pos2::new(half_width + radius + x, half_height + radius + y),
                        )
                    }
                    None => (
                        Pos2::new(size.x / 2.0 - radius, size.y / 2.0 - radius),
                        Pos2::new(size.x / 2.0 + radius, size.y / 2.0 + radius),
                    ),
                };
                Spinner::new().paint_at(ui, Rect::from_min_max(min, max));
            }

            if STATE.read().reload {
                STATE.write().reload = false;
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
