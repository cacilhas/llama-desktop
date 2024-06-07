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
    pub context: Vec<u16>,
}

impl State {
    pub fn reset(&mut self) {
        _eprintln!("RESETING STATE");
        self.input = "Why the sky is blue?".to_owned();
        self.title = String::new();
        self.output = String::new();
        self.retrieving = false;
        self.reload = true;
        self.context = Vec::new();
        _dbg!(self);
    }
}

pub fn set_model(model: impl Into<String>) -> bool {
    let model = model.into();
    _eprintln!("setting model to {}", &model);
    let models = STATE.read().models.clone();
    for (idx, model_) in models.iter().enumerate() {
        _dbg!(idx, model_);
        if model_.eq(&model) {
            _eprintln!("model {} found", &model);
            STATE.write().selected_model = idx;
            return true;
        }
    }
    _eprintln!("model {} not found", &model);
    return false;
}

#[dynamic]
pub static mut STATE: State = State {
    title: String::new(),
    models: Vec::new(),
    selected_model: usize::max_value(),
    input: "Why the sky is blue?".to_owned(),
    output: String::new(),
    retrieving: false,
    reload: true,
    escape: false,
    timeout_idx: usize::max_value(),
    context: Vec::new(),
};
