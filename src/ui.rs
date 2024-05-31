use eframe::Frame;
use eframe::*;
use egui::*;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use egui_extras::install_image_loaders;

#[derive(Debug)]
pub struct LlammaApp<'a> {
    logo: ImageSource<'a>,
    title_font: FontId,
    models: Vec<String>,
    selected_model: usize,
    input: String,
    output: String,
    cache: CommonMarkCache,
}

impl<'a> LlammaApp<'a> {
    pub fn new(cc: &CreationContext<'_>, fonts: FontDefinitions) -> Self {
        install_image_loaders(&cc.egui_ctx);
        cc.egui_ctx.set_fonts(fonts);
        Self {
            logo: include_image!("assets/logo.png"),
            title_font: FontId::new(32.0, FontFamily::Name("arial".into())),
            models: vec!["one".to_string(), "two".to_string(), "three".to_string()],
            selected_model: 0,
            input: "Why the sky is blue?".to_owned(),
            output: String::new(),
            cache: CommonMarkCache::default(),
        }
    }
}

impl<'a> App for LlammaApp<'a> {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        ctx.set_visuals(Visuals::dark());

        TopBottomPanel::top("title-panel")
            .exact_height(48.0)
            .show(ctx, |ui| {
                ui.columns(3, |uis| {
                    uis[0].with_layout(Layout::left_to_right(Align::Min), |ui| {
                        ui.add(
                            Image::new(self.logo.clone())
                                .fit_to_exact_size(Vec2 { x: 64.0, y: 64.0 }),
                        );
                    });

                    uis[1].with_layout(Layout::left_to_right(Align::Center), |ui| {
                        ui.label(
                            RichText::new("Llama Desktop")
                                .font(self.title_font.clone())
                                .strong(),
                        );
                    });

                    uis[2].with_layout(Layout::right_to_left(Align::Max), |ui| {
                        let models = &self.models;
                        ComboBox::from_id_source(Id::new("models"))
                            .selected_text(&models[self.selected_model])
                            .show_ui(ui, |ui| {
                                for (idx, opt) in models.iter().enumerate() {
                                    let value = ui.selectable_value(
                                        &mut self.selected_model,
                                        idx,
                                        opt.clone(),
                                    );
                                    if value.clicked() {
                                        self.selected_model = idx;
                                    }
                                }
                            });
                        ui.label(
                            RichText::new("Models:")
                                .font(self.title_font.clone())
                                .color(ecolor::Color32::from_rgb(0x54, 0x10, 0x21))
                                .strong(),
                        );
                    });
                });
            });

        CentralPanel::default().show(ctx, |ui| {
            let size = ui.available_size();
            let text_size = Vec2::new(size.x, size.y / 3.0);
            ui.add_sized(text_size, TextEdit::multiline(&mut self.input));

            ui.vertical_centered_justified(|ui| ui.button("send"));

            CommonMarkViewer::new("output").show(ui, &mut self.cache, &self.output);
        });
    }
}
