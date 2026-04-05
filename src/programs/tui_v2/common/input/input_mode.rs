use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, text::Line, Frame};

use crate::programs::tui_v2::common::placeholder::PlaceholderLine;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Mode {
    Insert,
    Normal,
    Readonly,
}

impl Mode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Mode::Insert => "Insert",
            Mode::Normal => "Normal",
            Mode::Readonly => "Readonly",
        }
    }

    pub fn is_insert(&self) -> bool {
        matches!(self, Self::Insert)
    }

    pub fn is_normal(&self) -> bool {
        matches!(self, Self::Normal)
    }

    pub fn is_readonly(&self) -> bool {
        matches!(self, Self::Readonly)
    }
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

    pub fn inner(&self) -> &str {
        &self.input
    }

    #[inline]
    pub fn is_editing(&self) -> bool {
        self.mode == Mode::Insert
    }

    pub fn has_selection(&self) -> bool {
        self.selection_start.is_some()
    }

    pub fn replace(&mut self, txt: &str) {
        self.input.extend(txt.chars());
        self.move_end_cursor();
    }

    pub fn set(&mut self, txt: &str) {
        self.input.clear();
        self.input.extend(txt.chars());
        self.move_end_cursor();
    }

    pub fn reset(&mut self) {
        if self.is_editing() {
            self.mode = Mode::Normal;
        }
        self.input.clear();
        self.cursor = 0;
        self.selection_start = None;
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
        self.cursor -= 1;
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
        if self.input.is_empty() {
            if let Some(ref placeholder) = self.placeholder {
                placeholder.draw(area, frame);
            }
        } else {
            frame.render_widget(Line::raw(&self.input), area);
        }
    }

    pub fn map_key_to_action(&self, key: KeyEvent) -> Option<InputAction> {
        match key.code {
            crossterm::event::KeyCode::Backspace if self.is_editing() => {
                Some(InputAction::DeleteChar)
            }
            crossterm::event::KeyCode::Left => Some(InputAction::NextChar),
            crossterm::event::KeyCode::Right => Some(InputAction::PrevChar),
            crossterm::event::KeyCode::End => Some(InputAction::CursorEnd),
            crossterm::event::KeyCode::Home => Some(InputAction::CursorStart),
            crossterm::event::KeyCode::Esc => {
                if self.is_editing() {
                    return Some(InputAction::SetMode(Mode::Normal));
                }
                None
            }
            crossterm::event::KeyCode::Char(ch) => match ch {
                'v' if self.mode != Mode::Insert => {
                    if self.has_selection() {
                        Some(InputAction::ClearSelection)
                    } else {
                        Some(InputAction::StartSelection)
                    }
                }
                'l' if self.mode != Mode::Insert => Some(InputAction::NextChar),
                'h' if self.mode != Mode::Insert => Some(InputAction::PrevChar),
                'd' if self.mode != Mode::Insert => {
                    if self.has_selection() {
                        Some(InputAction::DeleteSelection)
                    } else {
                        Some(InputAction::DeleteChar)
                    }
                }
                'i' if self.mode == Mode::Normal => Some(InputAction::SetMode(Mode::Insert)),
                _ => {
                    if self.is_editing() {
                        Some(InputAction::InsertChar(ch))
                    } else {
                        None
                    }
                }
            },
            _ => None,
        }
    }

    pub fn handle_action(&mut self, action: InputAction) {
        match action {
            InputAction::NextChar => self.next_char(),
            InputAction::PrevChar => self.prev_char(),
            InputAction::CursorStart => self.move_start_cursor(),
            InputAction::CursorEnd => self.move_end_cursor(),
            InputAction::DeleteChar => self.delete_char(),
            // InputAction::NextWord => self.nex,
            // InputAction::PrevWord => todo!(),
            InputAction::StartSelection => self.start_selection(),
            InputAction::DeleteSelection => self.delete_selection(),
            InputAction::ClearSelection => self.clear_selection(),
            InputAction::InsertChar(ch) => self.insert_char(ch),
            InputAction::Copy => todo!(),
            InputAction::Paste => todo!(),
            InputAction::SetMode(mode) => self.mode = mode,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<InputAction> {
        let action = self.map_key_to_action(key);
        if let Some(act) = action {
            self.handle_action(act);
        }

        action
    }
}

#[derive(Clone, Copy)]
pub enum InputAction {
    NextChar,
    PrevChar,
    CursorStart,
    CursorEnd,

    DeleteChar,
    // NextWord,
    // PrevWord,
    // CursorNext,
    // CursorPrev,
    StartSelection,
    DeleteSelection,
    ClearSelection,
    InsertChar(char),
    Copy,
    Paste,
    SetMode(Mode),
}

impl InputAction {
    pub fn is_mutation(&self) -> bool {
        // matches!(self, )
        match self {
            InputAction::NextChar => false,
            InputAction::PrevChar => false,
            InputAction::CursorStart => false,
            InputAction::CursorEnd => false,
            InputAction::DeleteChar => true,
            InputAction::StartSelection => false,
            InputAction::DeleteSelection => true,
            InputAction::ClearSelection => false,
            InputAction::InsertChar(_) => true,
            InputAction::Copy => false,
            InputAction::Paste => false,
            InputAction::SetMode(_) => false,
        }
    }
}
