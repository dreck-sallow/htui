use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    widgets::{Block, Clear, List, ListState},
    Frame,
};

use crate::store::models::HttpMethod;

use super::{action::Action, pane_state::PaneState};

const METHODS: [HttpMethod; 7] = [
    HttpMethod::Get,
    HttpMethod::Post,
    HttpMethod::Put,
    HttpMethod::Patch,
    HttpMethod::Delete,
    HttpMethod::Head,
    HttpMethod::Options,
];

struct MethodSelectorState {
    // method: HttpMethod,
    list_state: ListState,
}

impl MethodSelectorState {
    pub fn new() -> Self {
        Self {
            // method: HttpMethod::Get,
            list_state: ListState::default(),
        }
    }

    pub fn select(&mut self, method: HttpMethod) {
        let idx = METHODS
            .iter()
            .enumerate()
            .find(|(_, m)| **m == method)
            .map(|(i, _)| i)
            .unwrap();
        // self.method = method;
        self.list_state.select(Some(idx));
    }

    pub fn mut_state(&mut self) -> &mut ListState {
        &mut self.list_state
    }

    pub fn idx(&mut self) -> Option<usize> {
        self.list_state.selected()
    }

    pub fn next_method(&mut self) {
        self.list_state.select_next();
    }

    pub fn prev_method(&mut self) {
        self.list_state.select_previous();
    }
}

pub struct MethodSelectorView {
    state: MethodSelectorState,
    coord: Option<(u16, u16)>,
}

impl MethodSelectorView {
    pub fn new() -> Self {
        Self {
            state: MethodSelectorState::new(),
            coord: None,
        }
    }

    pub fn select_method(&mut self, method: HttpMethod) {
        self.state.select(method);
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        if let Some((x, y)) = self.coord {
            let area = Rect {
                x: x,
                y: y,
                width: 8 + 2,
                height: METHODS.len() as u16 + 2,
            };
            let list = List::new(METHODS.map(|m| Into::<&str>::into(&m)))
                .block(Block::bordered().border_style(Style::default().blue()))
                .highlight_style(Style::default().blue());

            frame.render_widget(Clear, area);

            frame.render_stateful_widget(list, area, self.state.mut_state());
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent, state: &mut PaneState, actions: &mut Vec<Action>) {
        if let KeyEventKind::Press = key.kind {
            match key.code {
                KeyCode::Char('j') | KeyCode::Down => self.state.next_method(),
                KeyCode::Char('k') | KeyCode::Up => self.state.prev_method(),
                KeyCode::Enter => {
                    // TODO: go back to method selector
                    // state.focus_element(super::focus::ElementFocus::RequestBuilder);
                    state.hidden_overlay();
                    let method = METHODS[self.state.idx().unwrap()];
                    actions.push(Action::SelectedMethod(method));
                }
                KeyCode::Esc => {
                    state.hidden_overlay();
                }
                _ => {}
            }
        }
    }

    pub fn handle_action(&mut self, action: Action) {
        match action {
            Action::SelectMethod(coord, http_method) => {
                self.coord = Some(coord);
                self.state.select(http_method);
            }
            _ => {}
        }
    }
}
