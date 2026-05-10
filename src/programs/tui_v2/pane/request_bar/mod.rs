use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    text::Span,
    Frame,
};
use select_method::SelectMethodPopup;

use crate::{
    programs::tui_v2::common::{elements::ui_block, input::input_mode::InputMode},
    store::models::HttpMethod,
};

use super::state_v2::{collections::ListIdx, PaneState, SectionFocus};

mod select_method;

pub struct RequestBar {
    method: HttpMethod,
    input_url: InputMode,
    select_method: SelectMethodPopup,
    show_select_method: bool,
}

impl RequestBar {
    pub fn new() -> Self {
        Self {
            method: HttpMethod::Get,
            input_url: InputMode::new(""),
            select_method: SelectMethodPopup::new(),
            show_select_method: false,
        }
    }

    pub fn sync(&mut self, state: &mut PaneState) {
        self.reset();

        if let ListIdx::Item(i, sub_i) = state.collections.idx {
            let request = &state.collections.get_requests(i).unwrap()[sub_i];
            self.input_url.replace(&request.url);
            self.method = request.method.clone();
        }
    }

    pub fn reset(&mut self) {
        self.input_url.reset();
        self.select_method.reset();
    }

    pub fn draw(&self, area: Rect, frame: &mut Frame, state: &PaneState) {
        let container = ui_block("", state.focus == SectionFocus::RequestBar);

        let [method_area, input_area, status_area] = Layout::horizontal([
            Constraint::Length(9),
            Constraint::Fill(1),
            Constraint::Length(6),
        ])
        .areas(container.inner(area));

        frame.render_widget(container, area);
        frame.render_widget(
            Span::raw(self.method.to_string().to_uppercase()),
            method_area,
        );
        self.input_url.draw(input_area, frame);
        frame.render_widget(Span::raw("SEND"), status_area);
    }

    pub fn draw_overlay(&self, _area: Rect, frame: &mut Frame) {
        if self.show_select_method {
            self.select_method.draw(frame);
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent, state: &mut PaneState) {
        if self.show_select_method {
            match key.code {
                crossterm::event::KeyCode::Enter if !self.select_method.is_editing() => {
                    if let Some(method) = self.select_method.selected() {
                        self.method = method;
                    }

                    self.show_select_method = false;
                    self.select_method.reset();
                }
                crossterm::event::KeyCode::Esc if !self.select_method.is_editing() => {
                    self.show_select_method = false;
                    self.select_method.reset();
                }
                _ => {
                    self.select_method.handle_key(key, state);
                }
            }
        } else {
            match key.code {
                // crossterm::event::KeyCode::Enter => {}
                // crossterm::event::KeyCode::Enter if key.modifiers == KeyModifiers::SHIFT => {
                crossterm::event::KeyCode::Enter => {
                    self.show_select_method = true;
                    self.select_method.select(&self.method);
                }
                crossterm::event::KeyCode::Tab => {
                    state.focus = SectionFocus::RequestBuilder;
                    if let ListIdx::Item(i, sub_i) = state.collections.idx {
                        let request = state.collections.get_request_mut((i, sub_i)).unwrap();
                        request.url = self.input_url.inner().to_string();
                        request.method = self.method.clone();
                    }
                }
                crossterm::event::KeyCode::BackTab => {
                    state.focus = SectionFocus::Collections;
                    if let ListIdx::Item(i, sub_i) = state.collections.idx {
                        let request = state.collections.get_request_mut((i, sub_i)).unwrap();
                        request.url = self.input_url.inner().to_string();
                        request.method = self.method.clone();
                    }
                }
                _ => {
                    self.input_url.handle_key(key);
                }
            }
        }
    }
}
