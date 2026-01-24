use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, text::Line, Frame};

use crate::programs::tui_v2::common::{elements::ui_block, placeholder::PlaceholderLine};

#[derive(PartialEq, Eq)]
pub enum Mode {
    Insert,
    Normal,
    Readonly,
}

pub struct InputMode {
    mode: Mode,
    input: String,
    cursor: usize,
    placeholder: Option<PlaceholderLine>,
    selection_start: Option<usize>,
}

impl InputMode {
    pub fn new<S: Into<String>>(s: S) -> Self {
        let input = s.into();
        let chars = input.chars().count();

        Self {
            mode: Mode::Normal,
            input,
            cursor: chars,
            placeholder: None,
            selection_start: None,
        }
    }

    pub fn with_placeholder(mut self, placeholder: PlaceholderLine) -> Self {
        self.placeholder = Some(placeholder);
        self
    }

    pub fn is_editing(&self) -> bool {
        self.mode == Mode::Insert
    }

    pub fn has_selection(&self) -> bool {
        self.selection_start.is_some()
    }

    pub fn next_char(&mut self) {
        let chars = self.input.chars().count();
        if chars > self.cursor {
            self.cursor += 1;
        }

        if let Some(start) = self.selection_start.as_mut() {
            if *start < chars.saturating_sub(1) {
                *start += 1;
            }
        }
    }

    pub fn prev_char(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }

        if let Some(start) = self.selection_start.as_mut() {
            if *start > 0 {
                *start -= 1;
            }
        }
    }

    pub fn delete_char(&mut self) {
        let chars = self.input.chars().count();
        if chars == 0 || self.cursor == 0 {
            return;
        }

        self.input.remove(self.cursor - 1);
    }

    pub fn move_start_cursor(&mut self) {
        self.cursor = 0;
        if let Some(start) = self.selection_start.as_mut() {
            *start = 0;
        }
    }

    pub fn move_end_cursor(&mut self) {
        let chars = self.input.chars().count();
        self.cursor = chars;
        if let Some(start) = self.selection_start.as_mut() {
            *start = chars;
        }
    }

    pub fn start_selection(&mut self) {
        let chars = self.input.chars().count();
        let bound_idx = self.cursor.min(chars - 1);
        self.selection_start = Some(bound_idx);
    }

    pub fn delete_selection(&mut self) {
        if let Some(start_selection) = self.selection_start {
            // Delete one char
            if start_selection == self.cursor {
                let mut chars = self.input.chars().count();
                self.input.remove(self.cursor.min(chars - 1));

                self.selection_start = None;
                chars -= 1;
                self.cursor = self.cursor.min(chars);
                return;
            }

            let bound_idx = self.cursor.min(self.input.chars().count() - 1);
            let start_idx = start_selection.min(bound_idx);
            let end_idx = start_selection.max(bound_idx) + 1;

            self.input.drain(start_idx..end_idx);
            self.cursor = self.cursor.min(self.input.chars().count());
        }
    }

    pub fn clear_selection(&mut self) {
        if self.selection_start.is_some() {
            self.selection_start = None;
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        self.input.insert(self.cursor, ch);
        self.cursor += 1;
    }
}

impl InputMode {
    pub fn draw(&self, area: Rect, frame: &mut Frame) {
        let block = ui_block("", false);
        let inner_area = block.inner(area);

        frame.render_widget(block, area);

        if self.input.is_empty() {
            if let Some(ref placeholder) = self.placeholder {
                placeholder.draw(inner_area, frame);
            }
        } else {
            frame.render_widget(Line::raw(&self.input), inner_area);
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            crossterm::event::KeyCode::Backspace => {
                self.delete_char();
            }
            crossterm::event::KeyCode::Left => {
                self.next_char();
            }
            crossterm::event::KeyCode::Right => {
                self.prev_char();
            }
            crossterm::event::KeyCode::End => {
                self.move_end_cursor();
            }
            crossterm::event::KeyCode::Home => {
                self.move_start_cursor();
            }
            // crossterm::event::KeyCode::Esc => todo!(),
            crossterm::event::KeyCode::Enter => {}
            crossterm::event::KeyCode::Char(ch) => {
                if ch == 'v' && self.mode != Mode::Insert {
                    if self.has_selection() {
                        self.clear_selection();
                    } else {
                        self.start_selection();
                    }
                } else if ch == 'l' && self.mode != Mode::Insert {
                    self.next_char();
                } else if ch == 'h' && self.mode != Mode::Insert {
                    self.prev_char();
                } else if ch == 'd' && self.mode != Mode::Insert {
                    if self.has_selection() {
                        self.delete_selection();
                    } else {
                        self.delete_char();
                    }
                } else {
                    self.insert_char(ch);
                }
            }
            // crossterm::event::KeyCode::Insert => todo!(),
            // crossterm::event::KeyCode::Delete => todo!(),
            // crossterm::event::KeyCode::Tab => todo!(),
            // crossterm::event::KeyCode::BackTab => todo!(),
            // crossterm::event::KeyCode::Up => todo!(),
            // crossterm::event::KeyCode::Down => todo!(),
            // crossterm::event::KeyCode::PageUp => todo!(),
            // crossterm::event::KeyCode::PageDown => todo!(),
            // crossterm::event::KeyCode::F(_) => todo!(),
            // crossterm::event::KeyCode::Null => todo!(),
            // crossterm::event::KeyCode::CapsLock => todo!(),
            // crossterm::event::KeyCode::ScrollLock => todo!(),
            // crossterm::event::KeyCode::NumLock => todo!(),
            // crossterm::event::KeyCode::PrintScreen => todo!(),
            // crossterm::event::KeyCode::Pause => todo!(),
            // crossterm::event::KeyCode::Menu => todo!(),
            // crossterm::event::KeyCode::KeypadBegin => todo!(),
            // crossterm::event::KeyCode::Media(media_key_code) => todo!(),
            // crossterm::event::KeyCode::Modifier(modifier_key_code) => todo!(),
            _ => {}
        }
    }
}

pub enum InputAction {
    NextChar,
    PrevChar,
    DeleteChar,
    NextWord,
    PrevWord,
    CursorStart,
    CursorEnd,
    // CursorNext,
    // CursorPrev,
    StartSelection,
    DeleteSelection,
    ClearSelection,
    InsertChar(char),
    Copy,
    Paste,
}
