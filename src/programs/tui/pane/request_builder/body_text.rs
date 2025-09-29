use ratatui::layout::Rect;

use crate::{
    app_project::models::BodyContent,
    programs::tui::{
        common::{Interactive, UiComposedElement},
        pane::text_editor::TextEditor,
    },
};

pub struct BodyTextEditor {
    render_area: Rect,
    editor: TextEditor,
}

impl BodyTextEditor {
    pub fn new() -> Self {
        Self {
            render_area: Rect::default(),
            editor: TextEditor::new(true),
        }
    }

    pub fn body_model(&self) -> BodyContent {
        BodyContent::Text(self.editor.lines().join("\n"))
    }

    pub fn set_data(&mut self, text: &str) {
        self.editor.clean_lines();
        self.editor.insert_str(text);
    }

    pub fn render_area(&self) -> Rect {
        self.render_area
    }
}

impl<'params> UiComposedElement<'params> for BodyTextEditor {
    type Params = ();

    fn set_area(&mut self, area: ratatui::prelude::Rect, _viewport_area: ratatui::prelude::Rect) {
        self.render_area = area;
    }

    fn draw(&self, _params: Self::Params, frame: &mut ratatui::Frame) {
        frame.render_widget(&self.editor, self.render_area);
    }
}

impl<'params> Interactive<'params> for BodyTextEditor {
    type Effect = ();
    type Params = ();

    fn is_input_focus(&self) -> bool {
        self.editor.mode().is_write_mode()
    }

    fn handle_key(
        &mut self,
        _params: Self::Params,
        key: crossterm::event::KeyEvent,
    ) -> Self::Effect {
        self.editor.handle_key(key);
    }
}
