use std::io::Result;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use super::elements::collections::{state::CollectionState, ui::CollectionsUi};

#[derive(Default)]
pub struct AppState {
    collections: CollectionState,
}

impl AppState {
    pub fn with_collections(mut self, collections: CollectionState) -> Self {
        self.collections = collections;
        self
    }
}

pub struct App {
    state: AppState,
}

impl App {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    pub fn render(&self, frame: &mut Frame) -> Result<()> {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
            .split(frame.area());

        CollectionsUi::render(frame, chunks[0], &self.state.collections);

        Ok(())
    }
}
