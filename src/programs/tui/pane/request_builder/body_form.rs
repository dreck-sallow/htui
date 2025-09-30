use arboard::Clipboard;
use ratatui::layout::Rect;

use crate::{
    app_project::models::{BodyContent, KeyValueParam},
    programs::tui::{
        common::{Interactive, UiComposedElement},
        config::Config,
    },
};

use super::params_table::ParamsTable;

pub struct BodyFormEditor {
    form: ParamsTable,
}

impl BodyFormEditor {
    pub fn new() -> Self {
        Self {
            form: ParamsTable::new(),
        }
    }

    pub fn body_model(&self) -> BodyContent {
        BodyContent::Form(self.form.get_data())
    }

    pub fn render_area(&self) -> Rect {
        self.form.render_area()
    }

    pub fn set_data(&mut self, params: Vec<KeyValueParam>) {
        self.form.set_state(params);
    }
}

impl<'params> UiComposedElement<'params> for BodyFormEditor {
    type Params = &'params Config;

    fn set_area(&mut self, area: Rect, viewport_area: Rect) {
        self.form.set_area(area, viewport_area);
        // self.render_area = area;
    }

    fn draw(&self, params: Self::Params, frame: &mut ratatui::Frame) {
        self.form.draw(params, frame);
    }

    fn draw_overlay(&self, params: Self::Params, frame: &mut ratatui::Frame) {
        self.form.draw_overlay(params, frame);
    }
}

impl<'params> Interactive<'params> for BodyFormEditor {
    type Effect = ();

    type Params = (&'params Config, &'params mut Clipboard);

    fn handle_key(
        &mut self,
        params: Self::Params,
        key: crossterm::event::KeyEvent,
    ) -> Self::Effect {
        self.form.handle_key(params, key)
    }
}
