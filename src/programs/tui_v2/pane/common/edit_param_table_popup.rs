use crossterm::event::KeyEvent;
use ratatui::{layout::Constraint, Frame};

use crate::programs::tui_v2::{
    common::overlays::popup_input::{CenterArea, PopupInput},
    pane::state::ParamsTable,
};

enum ParamField {
    Key,
    Value,
}

pub struct EditParamTablePopup {
    popup_input: PopupInput,
    param_field: ParamField,
}

impl EditParamTablePopup {
    pub fn new() -> Self {
        Self {
            popup_input: PopupInput::new(),
            param_field: ParamField::Key,
        }
    }

    pub fn is_visible(&self) -> bool {
        self.popup_input.is_visible()
    }

    pub fn edit_key(&mut self, value: &str) {
        self.param_field = ParamField::Key;
        self.popup_input.show(value);
    }

    pub fn edit_value(&mut self, value: &str) {
        self.param_field = ParamField::Value;
        self.popup_input.show(value);
    }

    pub fn hide(&mut self) {
        self.popup_input.hide();
    }
}

impl EditParamTablePopup {
    pub fn draw(&self, frame: &mut Frame) {
        let title = match self.param_field {
            ParamField::Key => " Edit key ",
            ParamField::Value => " Edit Value ",
        };

        self.popup_input.draw_center(
            title,
            CenterArea {
                height: Constraint::Length(3),
                width: Constraint::Percentage(40),
            },
            frame,
        );
    }

    pub fn handle_key(&mut self, key: KeyEvent, table: &mut ParamsTable) {
        let itm = table.current_mut().unwrap();

        match key.code {
            crossterm::event::KeyCode::Enter => {
                let text = self.popup_input.value().to_string();
                match self.param_field {
                    ParamField::Key => itm.key = text,
                    ParamField::Value => itm.value = text,
                }
                self.hide();
            }
            crossterm::event::KeyCode::Esc if !self.popup_input.is_editing() => {
                self.hide();
            }
            _ => {
                self.popup_input.handle_input_key(key);
            }
        }
    }
}
