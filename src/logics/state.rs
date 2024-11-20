#[derive(Debug)]
pub struct State {
    pub title: String,
    pub models: Vec<String>,
    pub selected_model: usize,
    pub input: String,
    pub output: String,
    pub retrieving: bool,
    pub reload: bool,
    pub timeout_idx: usize,
    pub escape: bool,
    pub context: Vec<u32>,
    pub cwd: String,
}

impl State {
    pub fn reset(&mut self) {
        warn!("RESETTING STATE");
        self.input = "Why the sky is blue?".to_owned();
        self.title = String::new();
        self.output = String::new();
        self.retrieving = false;
        self.reload = true;
        self.context = Vec::new();
        debug!(self);
    }
}

pub fn set_model(model: impl Into<String>) -> bool {
    let model = model.into();
    warn!("setting model to {}", &model);
    let models = STATE.read().models.clone();
    for (idx, model_) in models.iter().enumerate() {
        debug!(idx, model_);
        if model_.eq(&model) {
            warn!("model {} found", &model);
            STATE.write().selected_model = idx;
            return true;
        }
    }
    warn!("model {} not found", &model);
    false
}

#[dynamic]
pub static mut STATE: State = State {
    title: String::new(),
    models: Vec::new(),
    selected_model: usize::MAX,
    input: "Why the sky is blue?".to_owned(),
    output: String::new(),
    retrieving: false,
    reload: true,
    escape: false,
    timeout_idx: usize::MAX,
    context: Vec::new(),
    cwd: String::new(),
};
