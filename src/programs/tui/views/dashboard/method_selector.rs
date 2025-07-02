use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::Span,
    widgets::{Block, Borders, Clear},
    Frame,
};

use crate::{programs::tui::element_view::ElementView, store::models::HttpMethod};

use super::global_pane_state::GlobalPaneState;

const METHODS: [HttpMethod; 7] = [
    HttpMethod::Get,
    HttpMethod::Post,
    HttpMethod::Put,
    HttpMethod::Patch,
    HttpMethod::Delete,
    HttpMethod::Head,
    HttpMethod::Options,
];

fn current_method_idx(current: HttpMethod) -> usize {
    METHODS
        .iter()
        .enumerate()
        .find(|(_i, method)| **method == current)
        .unwrap()
        .0
}

fn next_method(current_idx: usize) -> usize {
    if current_idx >= METHODS.len() - 1 {
        current_idx
    } else {
        current_idx + 1
    }
}

fn prev_method(current_idx: usize) -> usize {
    if current_idx == 0 {
        current_idx
    } else {
        current_idx - 1
    }
}

pub struct MethodSelectorState {
    method: HttpMethod,
    area: Rect,
}

impl MethodSelectorState {
    pub fn new() -> Self {
        Self {
            method: HttpMethod::Get,
            area: Rect::default(),
        }
    }

    pub fn inner(&self) -> HttpMethod {
        self.method
    }

    pub fn set_state(&mut self, (method, area): (HttpMethod, (u16, u16))) {
        self.method = method;
        self.area = Rect {
            x: area.0,
            y: area.1,
            width: (Into::<&str>::into(&HttpMethod::Options).len() + 4) as u16,
            height: METHODS.len() as u16 + 2u16,
        };
    }
}

pub struct MethodSelectorView {
    // selected_idx: Option<usize>,
}

impl MethodSelectorView {
    pub fn new() -> Self {
        Self {}
    }
}

impl ElementView for MethodSelectorView {
    type State = GlobalPaneState;

    fn draw(&self, frame: &mut Frame, state: &Self::State) {
        let inner_state = state.method_selector_state_ref();
        let current_idx = current_method_idx(inner_state.method);

        frame.render_widget(Clear, inner_state.area);

        let block = Block::bordered()
            // .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
            .border_style(Style::default().blue());
        let area = block.inner(inner_state.area);

        frame.render_widget(block, inner_state.area);

        for (i, method) in METHODS.iter().enumerate() {
            let style = if current_idx == i {
                Style::default().on_blue()
            } else {
                Style::default()
            };

            let line_area = Rect {
                x: area.left(),
                y: (area.top() + (1u16 * i as u16)),
                width: area.width,
                height: 1,
            };

            frame.render_widget(
                Span::from(Into::<&str>::into(method)).style(style),
                line_area,
            );
        }
    }

    fn on_key(&mut self, key: KeyEvent, state: &mut Self::State) {
        let inner = state.method_selector_state_mut();

        if let KeyEventKind::Press = key.kind {
            match key.code {
                KeyCode::Char('j') | KeyCode::Down => {
                    let idx = next_method(current_method_idx(inner.method));
                    inner.method = METHODS[idx];
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    let idx = prev_method(current_method_idx(inner.method));
                    inner.method = METHODS[idx];
                }
                KeyCode::Enter => {
                    // TODO: go back to method selector
                    // state.focus_element(super::focus::ElementFocus::RequestBuilder);
                    state.hidden_overlay();
                    state.change_method_from_state();
                }
                KeyCode::Esc => {
                    state.hidden_overlay();
                }
                _ => {}
            }
        }
    }
}
