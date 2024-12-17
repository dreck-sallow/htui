use collections::CollectionsState;

pub mod collections;

pub struct AppState {
    collections: CollectionsState,
}

impl AppState {
    pub fn new(collections: CollectionsState) -> Self {
        Self { collections }
    }
}
