use crossterm::event::KeyEvent;
use ratatui::Frame;

use crate::programs::tui_v2::{
    common::{
        elements::{ui_block, utils::center_area},
        input::input_mode::{InputAction, InputMode},
    },
    pane::state::ParamsTable,
};

enum ParamField {
    Key,
    Value,
}

pub struct EditParamTablePopup {
    input_str: InputMode,
    show_edit_param: bool,
    param_field: ParamField,
}

impl EditParamTablePopup {
    pub fn new() -> Self {
        Self {
            input_str: InputMode::new(""),
            show_edit_param: false,
            param_field: ParamField::Key,
        }
    }

    pub fn is_visible(&self) -> bool {
        self.show_edit_param
    }

    pub fn value(&self) -> &str {
        self.input_str.inner()
    }

    pub fn is_key_field(&self) -> bool {
        matches!(self.param_field, ParamField::Key)
    }

    pub fn edit_key(&mut self, value: &str) {
        self.param_field = ParamField::Key;
        self.show_edit_param = true;
        self.input_str.replace(value);
    }

    pub fn edit_value(&mut self, value: &str) {
        self.param_field = ParamField::Value;
        self.show_edit_param = true;
        self.input_str.replace(value);
    }

    pub fn hide(&mut self) {
        self.show_edit_param = false;
        self.input_str.reset();
    }
}

impl EditParamTablePopup {
    pub fn draw(&self, frame: &mut Frame) {
        let area = center_area(
            frame.area(),
            ratatui::layout::Constraint::Length(3),
            ratatui::layout::Constraint::Percentage(40),
        );

        let title = match self.param_field {
            ParamField::Key => " Edit key ",
            ParamField::Value => " Edit Value ",
        };

        let block = ui_block(title, true);
        let inner_area = block.inner(area);

        frame.render_widget(block, area);
        self.input_str.draw(inner_area, frame);
    }

    pub fn handle_key(&mut self, key: KeyEvent, table: &mut ParamsTable) {
        let itm = table.current_mut().unwrap();

        match key.code {
            crossterm::event::KeyCode::Enter => {
                let text = self.input_str.inner().to_string();
                match self.param_field {
                    ParamField::Key => itm.key = text,
                    ParamField::Value => itm.value = text,
                }
                self.hide();
            }
            crossterm::event::KeyCode::Esc if !self.input_str.is_editing() => {
                self.hide();
            }
            _ => {
                self.input_str.handle_key(key);
            }
        }
    }

    pub fn handle_key_inner(&mut self, key: KeyEvent) -> Option<InputAction> {
        self.input_str.handle_key(key)
    }
}
