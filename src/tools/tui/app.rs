use std::io::Result;

use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use super::{
    core::elements::Element,
    elements::{
        collections::{CollectionState, Collections},
        url_iput::{state::UrlInputState, UrlInput},
    },
};

#[derive(Default)]
pub struct AppState {
    collections: CollectionState,
    url_input: UrlInputState,
}

impl AppState {
    pub fn with_collections(mut self, collections: CollectionState) -> Self {
        self.collections = collections;
        self
    }
}

pub struct App<'a> {
    state: AppState,
    url_input: UrlInput<'a>,
}

impl<'a> App<'a> {
    pub fn new(state: AppState) -> Self {
        let url_input = UrlInput::new();
        Self { state, url_input }
    }

    pub fn render(&self, frame: &mut Frame) -> Result<()> {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
            .split(frame.area());

        Collections::render(frame, chunks[0], &self.state.collections);

        let right_chunks = Layout::new(
            Direction::Vertical,
            [Constraint::Length(3), Constraint::Fill(1)],
        )
        .split(chunks[1]);

        self.url_input
            .render(frame, right_chunks[0], &self.state.url_input);

        // UrlInput::render(frame, right_chunks[0], &self.state.url_input);

        Ok(())
    }

    pub fn handle(&mut self, event: &KeyEvent) {
        Collections::event(&mut self.state.collections, event);
        self.url_input.event(&mut self.state.url_input, event);
    }
}
