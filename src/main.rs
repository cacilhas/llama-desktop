extern crate eframe;
extern crate egui_extras;
extern crate image;

mod fonts;
mod protocol;
mod ui;

use crate::fonts::initialize_fonts;
use crate::ui::LlammaApp;
use eframe::egui;
use std::error;

fn main() -> Result<(), Box<dyn error::Error>> {
    let viewport = egui::ViewportBuilder::default()
        .with_title("Llama Desktop")
        .with_inner_size([800.0, 1200.0]);

    let fonts = initialize_fonts();
    let options = eframe::NativeOptions {
        viewport,
        centered: true,
        run_and_return: false,
        ..Default::default()
    };

    eframe::run_native(
        "llama-desktop",
        options,
        Box::new(|cc| Box::new(LlammaApp::new(cc, fonts))),
    )?;

    Ok(())
}
