use ratatui::{
    layout::{Constraint, Layout},
    style::Style,
    text::Span,
    widgets::{Block, Borders},
};
use state::UrlInputState;
use tui_textarea::{Input, Key, TextArea};

use crate::tools::tui::core::elements::Element;

pub mod state;

pub struct UrlInput<'a> {
    text_area: TextArea<'a>,
}

impl<'a> UrlInput<'a> {
    pub fn new() -> Self {
        let mut textarea = TextArea::default();
        textarea.set_style(Style::default());
        textarea.insert_str("http://www.");
        textarea.set_block(Block::new().borders(Borders::ALL));

        Self {
            text_area: textarea,
        }
    }
}

impl<'a> Element for UrlInput<'a> {
    type State = UrlInputState;

    fn render(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
        _state: &Self::State,
    ) {
        frame.render_widget(&self.text_area, area);
    }

    fn event(&mut self, state: &mut Self::State, key: &crossterm::event::KeyEvent) {
        let input_event: Input = (*key).into();
        match input_event {
            Input {
                key: Key::Enter, ..
            } => state.set_url("".into()),
            input => {
                self.text_area.input(input);
                state.set_url(self.text_area.lines()[0].clone());
            }
        }
    }
}
