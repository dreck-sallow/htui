pub enum UpsertMethod {
    CreateRequest,
    CreateCollection,
    EditRequest,
    EditCollection,
}

pub struct UpsertItemState {
    method: UpsertMethod,
}

impl UpsertItemState {
    pub fn new() -> Self {
        Self {
            method: UpsertMethod::CreateRequest,
        }
    }

    pub fn set_method(&mut self, method: UpsertMethod) {
        self.method = method;
    }

    pub fn reset(&mut self) {
        self.method = UpsertMethod::CreateRequest
    }
}
