#[derive(Clone)]
pub enum UpsertMethod {
    CreateRequest(String),
    CreateCollection(String),
    EditRequest(String),
    EditCollection(String),
}

impl UpsertMethod {
    pub fn text(&self) -> &str {
        match self {
            UpsertMethod::CreateRequest(t) => t,
            UpsertMethod::CreateCollection(t) => t,
            UpsertMethod::EditRequest(t) => t,
            UpsertMethod::EditCollection(t) => t,
        }
    }
}

pub struct UpsertItemState {
    method: UpsertMethod,
}

impl UpsertItemState {
    pub fn new() -> Self {
        Self {
            method: UpsertMethod::CreateRequest(String::new()),
        }
    }

    pub fn method(&self) -> UpsertMethod {
        self.method.clone()
    }

    pub fn set_method(&mut self, method: UpsertMethod) {
        self.method = method;
    }

    pub fn reset(&mut self) {
        self.method = UpsertMethod::CreateRequest(String::new())
    }
}
