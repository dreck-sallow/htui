use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Rect},
    widgets::Clear,
    Frame,
};

use crate::programs::tui_v2::common::{
    elements::{ui_block, utils::center_area},
    input::input_mode::InputMode,
};

pub struct PopupInput {
    visible: bool,
    input: InputMode,
}

impl PopupInput {
    pub fn new() -> Self {
        Self {
            visible: false,
            input: InputMode::new(""),
        }
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn show(&mut self, input: &str) {
        self.visible = true;
        self.input.replace(input);
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.input.reset();
    }

    pub fn value(&self) -> &str {
        self.input.inner()
    }

    pub fn is_editing(&self) -> bool {
        self.input.is_editing()
    }
}

impl PopupInput {
    pub fn draw(&self, title: &str, area: Rect, frame: &mut Frame) {
        let block = ui_block(title, true);
        let inner_area = block.inner(area);

        frame.render_widget(Clear, area);
        frame.render_widget(block, area);
        self.input.draw(inner_area, frame);
    }

    pub fn draw_center(&self, title: &str, area: CenterArea, frame: &mut Frame) -> Rect {
        let area = center_area(frame.area(), area.height, area.width);
        self.draw(title, area, frame);
        area
    }

    pub fn handle_input_key(
        &mut self,
        key: KeyEvent,
    ) -> Option<crate::programs::tui_v2::common::input::input_mode::InputAction> {
        self.input.handle_key(key)
    }
}

pub struct CenterArea {
    pub height: Constraint,
    pub width: Constraint,
}
