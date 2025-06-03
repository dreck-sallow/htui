use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    text::Span,
    widgets::{Block, Tabs},
    Frame,
};
use request_builder_state::{RequestBuilderState, ViewTab};
use tui_textarea::TextArea;

use super::pane_state::PaneState;

mod request_builder_state;

pub struct RequestBuilderView {
    state: RequestBuilderState,
    url_input: TextArea<'static>,
}

impl RequestBuilderView {
    pub fn new() -> Self {
        Self {
            state: RequestBuilderState::new(),
            url_input: TextArea::default(),
        }
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered().title(" Request builder ");

        let [method_area, input_area, tabs_area, pane_area] = {
            let [header_area, content_area] =
                Layout::vertical([Constraint::Length(1), Constraint::Fill(1)])
                    .areas(block.inner(area));

            let [method_area, input_area] =
                Layout::horizontal([Constraint::Length(6), Constraint::Fill(1)]).areas(header_area);

            let [tabs_area, pane_area] =
                Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(content_area);

            [method_area, input_area, tabs_area, pane_area]
        };

        frame.render_widget(block, area);

        frame.render_widget(Span::from(self.state.method), method_area);
        frame.render_widget(&self.url_input, input_area);

        let tabs = Tabs::new([ViewTab::Headers.to_string(), ViewTab::Body.to_string()]).select(0);

        frame.render_widget(tabs, tabs_area);
    }

    pub fn handle_key(&mut self, state: &mut PaneState, key: KeyEvent) {}
}
