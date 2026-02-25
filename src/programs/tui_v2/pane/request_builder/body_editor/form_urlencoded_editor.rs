use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, Frame};

use crate::programs::tui_v2::pane::{common::params_table::ParamsTableUi, state::ParamsTable};

pub struct FormUrlEncodedEditor {
    params_table: ParamsTableUi,
    state: ParamsTable,
}

impl Default for FormUrlEncodedEditor {
    fn default() -> Self {
        Self::new(ParamsTable::default())
    }
}

impl FormUrlEncodedEditor {
    pub fn new(state: ParamsTable) -> Self {
        Self {
            params_table: ParamsTableUi::new(),
            state,
        }
    }
}

impl FormUrlEncodedEditor {
    pub fn draw(&self, area: Rect, frame: &mut Frame) {
        self.params_table.draw(&self.state, area, frame);
    }

    pub fn draw_overlay(&self, frame: &mut Frame) {
        self.params_table.draw_overlay(frame);
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        self.params_table.handle_table_key(key, &mut self.state);
    }
}
