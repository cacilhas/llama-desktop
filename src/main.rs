#[macro_use]
extern crate static_init;

mod fonts;
mod helpers;
mod logics;
mod ollama;
mod protocol;
mod ui;

use crate::fonts::initialize_fonts;
use crate::ui::LlamaApp;
use eframe::egui;

fn main() {
    let viewport = egui::ViewportBuilder::default()
        .with_title("Llama Desktop")
        .with_inner_size([800.0, 1200.0]);

    ollama::init();
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
        Box::new(|cc| Box::new(LlamaApp::new(cc, fonts))),
    )
    .unwrap();
}
