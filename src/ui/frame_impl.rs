use super::BoxLayout;
use super::RUNTIME;
use crate::fonts::set_font_size;
use crate::logics::Sender;
use crate::logics::*;
use eframe::Frame;
use eframe::*;
use egui::*;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

impl App for super::LlamaApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        ctx.set_visuals(Visuals::dark());
        let mut send_clicked = false;
        let mut quit_clicked = false;
        let mut new_clicked = false;
        let mut load_clicked = false;
        let mut save_clicked = false;

        set_font_size(ctx, 20.0);
        self.setup(frame);
        let retrieving = STATE.read().retrieving;

        TopBottomPanel::top("header")
            .exact_height(48.0)
            .show(ctx, |ui| {
                ui.columns(3, |cols| {
                    cols[0].with_layout(Layout::left_to_right(Align::Center), |ui| {
                        ui.menu_button("File", |ui| {
                            if retrieving {
                                let _ = ui.label(RichText::new("Load").weak());
                                let _ = ui.label(RichText::new("Save").weak());
                            } else {
                                load_clicked = Button::new(RichText::new("Load").strong())
                                    .shortcut_text(&format!("{}O", CMD))
                                    .ui(ui)
                                    .clicked();

                                if STATE.read().output.is_empty() {
                                    let _ = ui.label(RichText::new("Save").weak());
                                } else {
                                    save_clicked = Button::new(RichText::new("Save").strong())
                                        .shortcut_text(&format!("{}S", CMD))
                                        .ui(ui)
                                        .clicked();
                                }
                            }

                            ui.separator();

                            quit_clicked = Button::new(RichText::new("Quit").strong())
                                .shortcut_text(&format!("{}Q", CMD))
                                .ui(ui)
                                .clicked();
                        });
                        ui.menu_button("Actions", |ui| {
                            if retrieving {
                                let _ = ui.label(RichText::new("New").weak());
                                let _ = ui.label(RichText::new("Send").weak());
                            } else {
                                new_clicked = Button::new(RichText::new("New").strong())
                                    .shortcut_text(&format!("{}N", CMD))
                                    .ui(ui)
                                    .clicked();

                                send_clicked = Button::new(RichText::new("Send").strong())
                                    .shortcut_text(&format!("{}Enter", CMD))
                                    .ui(ui)
                                    .clicked();
                            }
                        });
                    });

                    cols[1].with_layout(Layout::top_down_justified(Align::Center), |ui| {
                        ui.horizontal(|ui| {
                            ui.add(
                                Image::new(self.logo.clone())
                                    .fit_to_exact_size(Vec2 { x: 48.0, y: 48.0 }),
                            );

                            ui.with_layout(Layout::left_to_right(Align::Max), |ui| {
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
                        });
                    });

                    cols[2].with_layout(Layout::right_to_left(Align::Center), |ui| {
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
                                let value = ui.selectable_value(&mut selected, idx, opt.to_owned());
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
            .exact_height(32.0)
            .show(ctx, |ui| {
                ui.columns(16, |cols| {
                    cols[1].with_layout(Layout::right_to_left(Align::Min), |ui| {
                        let text = RichText::new("Timeout:").strong();
                        if retrieving {
                            ui.label(format!("{}s", TIMEOUTS[STATE.read().timeout_idx]));
                            ui.label(text);
                        } else {
                            let mut state = STATE.write();
                            ComboBox::from_label(text)
                                .selected_text(format!("{}s", TIMEOUTS[state.timeout_idx]))
                                .show_ui(ui, |ui| {
                                    let mut idx = state.timeout_idx;
                                    for (i, tm) in TIMEOUTS.iter().enumerate() {
                                        let value =
                                            ui.selectable_value(&mut idx, i, format!("{}s", tm));
                                        if value.clicked() {
                                            idx = i;
                                        }
                                    }
                                    if idx != state.timeout_idx {
                                        state.timeout_idx = idx;
                                        if let Some(storage) = frame.storage_mut() {
                                            storage.set_string(
                                                "timeout",
                                                format!("{}", TIMEOUTS[idx]),
                                            );
                                            storage.flush();
                                        }
                                    }
                                });
                        }
                    });

                    cols[14].with_layout(Layout::right_to_left(Align::Center), |ui| {
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

                    cols[15].with_layout(Layout::left_to_right(Align::Center), |ui| {
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
            });

        CentralPanel::default().show(ctx, |ui| {
            let size = ui.available_size();
            let mut body: Option<Rect> = None;
            let mut input: Option<Response> = None;

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
                                let _ = input.insert(ui.add_sized(
                                    text_size,
                                    TextEdit::multiline(&mut STATE.write().input),
                                ));
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
                            let _ = input.insert(ui.add_sized(
                                text_size,
                                TextEdit::multiline(&mut STATE.write().input),
                            ));
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

            if retrieving {
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
                if let Some(storage) = frame.storage_mut() {
                    storage.set_string("cwd", STATE.read().cwd.to_owned());
                    storage.flush();
                }
                if let Some(input) = input {
                    input.request_focus();
                }
            }
        });

        if retrieving {
            if ctx.input(|st| st.key_pressed(Key::Escape)) {
                STATE.write().escape = true;
            }
        } else {
            if new_clicked || ctx.input(|rd| rd.modifiers.command && rd.key_pressed(Key::N)) {
                STATE.write().reset();
            }
            if load_clicked || ctx.input(|rd| rd.modifiers.command && rd.key_pressed(Key::O)) {
                RUNTIME.spawn(storage::load());
            }
            if save_clicked
                || (!STATE.read().output.is_empty()
                    && ctx.input(|rd| rd.modifiers.command && rd.key_pressed(Key::S)))
            {
                RUNTIME.spawn(storage::save_content(STATE.read().output.to_owned()));
            }
            if send_clicked || ctx.input(|rd| rd.modifiers.command && rd.key_pressed(Key::Enter)) {
                RUNTIME.spawn(Sender::default().send());
            }
        }

        if quit_clicked || ctx.input(|rd| rd.modifiers.command && rd.key_pressed(Key::Q)) {
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }
    }
}

#[dynamic]
static mut MD_CACHE: CommonMarkCache = CommonMarkCache::default();

#[cfg(not(target_os = "macos"))]
static CMD: &str = "Ctrl+";

#[cfg(target_os = "macos")]
static CMD: &str = "⌘";
