use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    text::Span,
    Frame,
};

use crate::{
    programs::tui_v2::common::{elements::ui_block, input::input_mode::InputMode},
    store::models::HttpMethod,
};

use super::state::{ListIdx, PaneState, SectionFocus};

pub struct RequestBar {
    method: String,
    input_url: InputMode,
}

impl RequestBar {
    pub fn new() -> Self {
        Self {
            method: HttpMethod::Get.to_string(),
            input_url: InputMode::new(""),
        }
    }

    pub fn sync(&mut self, state: &mut PaneState) {
        self.reset();

        if let super::state::ListIdx::Item(i, sub_i) = state.collections.idx {
            let request = &state.collections.items[i].requests[sub_i];
            self.input_url.replace(&request.url);
            self.method = request.method.to_string();
        }
    }

    pub fn reset(&mut self) {
        self.input_url.reset();
    }

    pub fn draw(&self, area: Rect, frame: &mut Frame, state: &PaneState) {
        let container = ui_block("", state.focus == SectionFocus::RequestBar);

        let [method_area, input_area, status_area] = Layout::horizontal([
            Constraint::Length(7),
            Constraint::Fill(1),
            Constraint::Length(6),
        ])
        .areas(container.inner(area));

        frame.render_widget(container, area);
        frame.render_widget(Span::raw(self.method.to_uppercase()), method_area);
        self.input_url.draw(input_area, frame);
        frame.render_widget(Span::raw("SEND"), status_area);
    }

    pub fn handle_key(&mut self, key: KeyEvent, state: &mut PaneState) {
        match key.code {
            // crossterm::event::KeyCode::Enter => {}
            // crossterm::event::KeyCode::Tab => {}
            crossterm::event::KeyCode::BackTab => {
                state.focus = SectionFocus::Collections;
                if let ListIdx::Item(i, sub_i) = state.collections.idx {
                    let request = &mut state.collections.items[i].requests[sub_i];
                    request.url = self.input_url.inner().to_string();
                }
            }
            // crossterm::event::KeyCode::Char(_) => {}
            _ => {
                self.input_url.handle_key(key);
            }
        }
    }
}
