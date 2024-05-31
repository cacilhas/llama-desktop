use eframe::Frame;
use eframe::*;
use egui::*;
use egui_extras::install_image_loaders;

pub struct LlammaApp<'a> {
    logo: ImageSource<'a>,
    title_font: FontId,
    model_font: FontId,
}

impl<'a> LlammaApp<'a> {
    pub fn new(cc: &CreationContext<'_>, fonts: FontDefinitions) -> Self {
        install_image_loaders(&cc.egui_ctx);
        cc.egui_ctx.set_fonts(fonts);
        Self {
            logo: include_image!("assets/logo.png"),
            title_font: FontId::new(42.0, FontFamily::Proportional),
            model_font: FontId::new(32.0, FontFamily::Name("arial".into())),
        }
    }
}

impl<'a> App for LlammaApp<'a> {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        let size = ctx.available_rect();
        TopBottomPanel::top("title")
            .exact_height(48.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.set_width(size.width());
                    match ui.data(|tm| tm.get_temp::<f32>(Id::new("title-size"))) {
                        Some(value) => {
                            ui.style_mut().spacing.item_spacing = Vec2 {
                                x: (size.width() - value) / 3.0,
                                y: 0.0,
                            }
                        }
                        None => (),
                    }

                    ui.style_mut().visuals.dark_mode = true;

                    // Widgets
                    let r1 = ui.add(
                        Image::new(self.logo.clone()).fit_to_exact_size(Vec2 { x: 64.0, y: 64.0 }),
                    );
                    let r2 = ui.label(
                        RichText::new("Llama Desktop")
                            .font(self.title_font.clone())
                            .strong(),
                    );
                    let r3 = ui.label(
                        RichText::new("Models:")
                            .font(self.model_font.clone())
                            .color(ecolor::Color32::from_rgb(0x54, 0x10, 0x21))
                            .strong(),
                    );

                    ui.data_mut(|tm| {
                        tm.insert_temp(
                            Id::new("title-size"),
                            r1.rect.width() + r2.rect.width() + r3.rect.width(),
                        )
                    });
                })
            });
        CentralPanel::default().show(ctx, |ui| {
            ui.style_mut().visuals.dark_mode = true;
        });
    }
}
