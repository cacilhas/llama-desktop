mod app_impl;
mod frame_impl;

use eframe::egui::{FontId, ImageSource};
use tokio::runtime::Runtime;

#[derive(Debug)]
pub struct LlamaApp {
    logo: ImageSource<'static>,
    horizontal: ImageSource<'static>,
    vertical: ImageSource<'static>,
    title_font: FontId,
    small_font: FontId,
    box_layout: BoxLayout,
    setupdone: bool,
}

#[derive(Debug, Default, Eq, PartialEq)]
pub(self) enum BoxLayout {
    Horizontally,
    Vertically,
    #[default]
    NotSet,
}

#[dynamic]
static RUNTIME: Runtime = Runtime::new().unwrap();
