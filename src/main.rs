extern crate eframe;
extern crate egui_extras;
extern crate image;

mod protocol;
mod ui;

use eframe::egui;
use std::error;
use ui::LlammaApp;

fn main() -> Result<(), Box<dyn error::Error>> {
    let viewport = egui::ViewportBuilder::default()
        .with_title("Llama Desktop")
        .with_inner_size([800.0, 1200.0]);

    let options = eframe::NativeOptions {
        viewport,
        centered: true,
        run_and_return: false,
        ..Default::default()
    };

    let font1 = egui::FontData::from_static(include_bytes!("assets/aclonica.ttf"));
    let font2 = egui::FontData::from_static(include_bytes!("assets/bellota.ttf"));
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert("arial".into(), font1);
    fonts.font_data.insert("sans".into(), font2);
    fonts
        .families
        .insert(egui::FontFamily::Name("arial".into()), vec!["arial".into()]);
    fonts
        .families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .insert(0, "sans".into());

    eframe::run_native(
        "llama-desktop",
        options,
        Box::new(|cc| Box::new(LlammaApp::new(cc, fonts))),
    )?;

    Ok(())
}
