use eframe::egui;
use egui_extras::install_image_loaders;

pub struct LlammaApp<'a> {
    logo: egui::ImageSource<'a>,
}

impl<'a> LlammaApp<'a> {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        install_image_loaders(&cc.egui_ctx);
        Self {
            logo: egui::include_image!("assets/logo.png"),
        }
    }
}

impl<'a> eframe::App for LlammaApp<'a> {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.style_mut().visuals.dark_mode = true;

            ui.horizontal(|ui| {
                ui.image(self.logo.clone());
            });
        });
    }
}
