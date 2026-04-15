use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, Frame};

use crate::programs::tui_v2::common::text_editor::TextEditor as Editor;

pub struct TextEditor {
    editor: Editor,
}

impl TextEditor {
    pub fn new() -> Self {
        Self {
            editor: Editor::new(true),
        }
    }

    pub fn set_content(&mut self, text: &str) {
        self.editor
            .set_mode(crate::programs::tui_v2::common::input::input_mode::Mode::Normal);
        self.editor.clean_lines();
        self.editor.insert_str(text);
    }

    pub fn value(&self) -> String {
        self.editor.lines().join("\n")
    }
}

impl TextEditor {
    pub fn draw(&self, area: Rect, frame: &mut Frame) {
        frame.render_widget(&self.editor, area);
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        self.editor.handle_key(key);
    }

    pub fn is_editing(&self) -> bool {
        self.editor.mode().is_insert()
    }
}
