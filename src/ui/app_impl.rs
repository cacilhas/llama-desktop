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
            temperature: 0.75,
            last_temperature: 0.75,
            setupdone: false,
        }
    }

    pub(super) fn setup(&mut self, frame: &mut Frame) {
        if self.setupdone {
            return;
        }
        warn!("running setup");
        if let Some(storage) = frame.storage() {
            self.setup_model(storage);
            self.setup_timeout(storage);
            self.setup_layout(storage);
            self.setup_cwd(storage);
            self.setup_temperature(storage);
        } else {
            let mut state = STATE.write();
            state.selected_model = 0;
            state.timeout_idx = 1;
            state.cwd = env!["HOME"].to_string();
            self.box_layout = BoxLayout::Vertically;
        }
        self.setupdone = true;
        debug!(self);
    }

    fn setup_model(&mut self, storage: &dyn Storage) {
        let selected_model = storage
            .get_string("selected-model")
            .unwrap_or("0".to_string());
        let idx: usize = selected_model.parse().unwrap_or(0);
        if idx < STATE.read().models.len() {
            STATE.write().selected_model = idx;
        } else {
            STATE.write().selected_model = 0;
        }
    }

    fn setup_timeout(&mut self, storage: &dyn Storage) {
        let timeout: usize = storage
            .get_string("timeout")
            .unwrap_or("20".to_string())
            .parse()
            .unwrap_or(20);
        for (idx, tm) in TIMEOUTS.iter().enumerate() {
            if *tm == timeout {
                STATE.write().timeout_idx = idx;
                return;
            }
        }
        STATE.write().timeout_idx = 1;
    }

    fn setup_layout(&mut self, storage: &dyn Storage) {
        if storage.get_string("layout").unwrap_or("V".to_string()) == *"H" {
            self.box_layout = BoxLayout::Horizontally;
        } else {
            self.box_layout = BoxLayout::Vertically;
        }
    }

    fn setup_cwd(&mut self, storage: &dyn Storage) {
        STATE.write().cwd = storage
            .get_string("cwd")
            .unwrap_or(env!["HOME"].to_string());
    }

    fn setup_temperature(&mut self, storage: &dyn Storage) {
        self.temperature = storage
            .get_string("temperature")
            .unwrap_or("0.75".to_string())
            .parse()
            .unwrap_or(0.75);
        self.last_temperature = self.temperature;
    }
}
