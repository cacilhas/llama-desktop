use std::{thread, time::Duration};

use super::{BoxLayout, LlamaApp, RUNTIME};
use crate::logics::{STATE, TIMEOUTS};
use crate::ollama;
use eframe::Frame;
use eframe::*;
use egui::*;
use egui_extras::install_image_loaders;
use tokio::sync::oneshot;

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
            logo: include_image!("../assets/logo.png"),
            horizontal: include_image!("../assets/horizontal.png"),
            vertical: include_image!("../assets/vertical.png"),
            title_font: FontId::new(32.0, FontFamily::Name("arial".into())),
            small_font: FontId::new(12.0, FontFamily::Name("arial".into())),
            box_layout: BoxLayout::default(),
        }
    }

    pub(super) fn setup(&mut self, frame: &mut Frame) {
        if let Some(storage) = frame.storage() {
            self.setup_model(storage);
            self.setup_timeout(storage);
            self.setup_layout(storage);
        } else {
            let mut state = STATE.write();
            state.selected_model = 0;
            state.timeout_idx = 1;
            self.box_layout = BoxLayout::Vertically;
        }
    }

    fn setup_model(&mut self, storage: &dyn Storage) {
        if STATE.read().selected_model > STATE.read().models.len() {
            let selected_model = storage
                .get_string("selected-model")
                .unwrap_or("0".to_string());
            STATE.write().selected_model = selected_model.parse().unwrap_or(0);
        }
    }

    fn setup_timeout(&mut self, storage: &dyn Storage) {
        if STATE.read().timeout_idx > TIMEOUTS.len() {
            let timeout: usize = storage
                .get_string("timeout")
                .unwrap_or("20".to_string())
                .parse()
                .unwrap_or(20);
            for (idx, tm) in TIMEOUTS.iter().enumerate() {
                if *tm == timeout {
                    STATE.write().timeout_idx = idx;
                    break;
                }
            }
        }
    }

    fn setup_layout(&mut self, storage: &dyn Storage) {
        if self.box_layout == BoxLayout::NotSet {
            if storage.get_string("layout").unwrap_or("V".to_string()) == "H".to_string() {
                self.box_layout = BoxLayout::Horizontally;
            } else {
                self.box_layout = BoxLayout::Vertically;
            }
        }
    }
}
