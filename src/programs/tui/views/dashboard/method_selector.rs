use std::{cell::RefCell, rc::Rc};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    style::{Style, Stylize},
    Frame,
};

use crate::{
    programs::tui::{
        element_view::{Drawable, ElementView, Interactive},
        elements::dropdown::{OverlayDropdown, OverlayDropdownData, OverlayDropdown_v2},
    },
    store::models::HttpMethod,
};

use super::{
    global_pane_state::GlobalPaneState,
    pane_state::{history::MutationCollector, PaneState},
};

const METHODS: [HttpMethod; 7] = [
    HttpMethod::Get,
    HttpMethod::Post,
    HttpMethod::Put,
    HttpMethod::Patch,
    HttpMethod::Delete,
    HttpMethod::Head,
    HttpMethod::Options,
];

pub struct MethodSelectorView {
    inner: OverlayDropdown<HttpMethod>,
}

impl MethodSelectorView {
    pub fn new(shared: Rc<RefCell<OverlayDropdownData<HttpMethod>>>) -> Self {
        Self {
            inner: OverlayDropdown::with_items(shared, METHODS)
                .with_highlight_style(Style::default().on_light_blue()),
        }
    }
}

impl<'a> ElementView<'a> for MethodSelectorView {
    type State = GlobalPaneState;
    type Collector = MutationCollector<'a>;

    fn draw(&self, frame: &mut Frame, _state: &Self::State) {
        self.inner.draw(frame);
    }

    fn on_key(&mut self, key: KeyEvent, state: &mut Self::Collector) {
        if let KeyEventKind::Press = key.kind {
            match key.code {
                KeyCode::Char('j') | KeyCode::Down => {
                    self.inner.next();
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.inner.prev();
                }
                KeyCode::Enter => {
                    // state.hidden_overlay();
                    // state.set_current_request_method(self.inner.selected());
                }
                KeyCode::Esc => {
                    // state.hidden_overlay();
                }
                _ => {}
            }
        }
    }
}
