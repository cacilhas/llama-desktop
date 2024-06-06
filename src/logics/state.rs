#[derive(Debug)]
pub struct State {
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
        self.output = String::new();
        self.retrieving = false;
        self.reload = true;
        self.context = Vec::new();
        _dbg!(self);
    }
}

pub fn set_model(model: impl Into<String>) -> bool {
    let model = model.into();
    for (idx, model_) in STATE.read().models.iter().enumerate() {
        if *model_ == model {
            STATE.write().selected_model = idx;
            return true;
        }
    }
    return false;
}

#[dynamic]
pub static mut STATE: State = State {
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
