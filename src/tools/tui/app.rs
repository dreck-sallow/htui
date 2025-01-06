use std::io::Result;

use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use super::elements::collections::{CollectionState, Collections};

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

        Collections::render(frame, chunks[0], &self.state.collections);

        Ok(())
    }

    pub fn handle(&mut self, event: &KeyEvent) {
        Collections::event(&mut self.state.collections, event);
    }
}
