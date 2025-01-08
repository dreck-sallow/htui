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
        state: &Self::State,
    ) {
        let chunks = Layout::new(
            ratatui::layout::Direction::Horizontal,
            [Constraint::Length(10), Constraint::Fill(1)],
        )
        .split(area);

        let method_block = Block::new().borders(Borders::ALL);

        let span = Span::from(state.method().to_string());

        frame.render_widget(span, method_block.inner(area));
        frame.render_widget(method_block, chunks[0]);

        frame.render_widget(&self.text_area, chunks[1]);
    }

    fn event(&mut self, state: &mut Self::State, key: &crossterm::event::KeyEvent) {
        let input_event: Input = (*key).into();
        match input_event {
            Input {
                key: Key::Enter, ..
            } => state.set_url("".into()),
            input => {
                self.text_area.input(input);
            }
        }
    }
}
