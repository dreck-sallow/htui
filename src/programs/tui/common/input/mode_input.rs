use std::io::stdout;

use crossterm::{
    cursor::SetCursorStyle,
    event::{KeyCode, KeyEvent},
    execute,
};
use ratatui::{
    layout::{Position, Rect},
    style::{Style, Stylize},
    Frame,
};

use crate::programs::tui::common::UiElement;

use super::input::{EditHandler, Input};

#[derive(Debug, Clone, Copy, Default)]
pub enum InputMode {
    Write,
    #[default]
    Read,
    ReadOnly,
}

impl InputMode {
    pub fn is_read_mode(&self) -> bool {
        matches!(self, Self::Read)
    }

    pub fn is_write_mode(&self) -> bool {
        matches!(self, Self::Write)
    }

    pub fn is_read_only_mode(&self) -> bool {
        matches!(self, Self::ReadOnly)
    }
}

impl Into<&'static str> for &InputMode {
    fn into(self) -> &'static str {
        match self {
            InputMode::Write => "Write",
            InputMode::Read => "Read",
            InputMode::ReadOnly => "Read-Only",
        }
    }
}

pub struct ModeInput {
    mode: InputMode,
    input: Input,
    // suggestion: Option<String>,
    scroll_offset: usize,
    _render_area: Rect,
}

impl ModeInput {
    pub fn new(input: &str) -> Self {
        Self {
            mode: InputMode::default(),
            input: Input::from(input),
            scroll_offset: 0,
            // suggestion: None,
            _render_area: Rect::default(),
        }
    }

    // pub fn new_empty() -> Self {
    //     Self {
    //         mode: InputMode::default(),
    //         input: Input::default(),
    //         scroll_offset: 0,
    //         // suggestion: None,
    //         _render_area: Rect::default(),
    //     }
    // }

    pub fn replace(&mut self, input: &str) {
        self.input.replace(input);
    }

    pub fn mode(&self) -> InputMode {
        self.mode
    }

    pub fn txt(&self) -> &str {
        self.input.txt()
    }

    pub fn clear(&mut self) {
        self.input.clear();
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        let mut edit_handler = InputModeEditHandler {
            key,
            mode: self.mode,
        };

        self.input.edit(&mut edit_handler);
        self.mode = edit_handler.mode;

        // Compute the scroll_offset
        let right_end = self.scroll_offset + (self._render_area.width.saturating_sub(2) as usize);

        if right_end < (self.input.cursor().saturating_sub(1)) {
            self.scroll_offset = self.scroll_offset + (self.input.cursor() - right_end);
        } else if self.input.cursor().saturating_sub(1) < self.scroll_offset {
            self.scroll_offset =
                self.scroll_offset - (self.scroll_offset - self.input.cursor().saturating_sub(1));
        }
    }
}

impl UiElement for ModeInput {
    type Params = ();

    fn set_area(&mut self, area: Rect) {
        self._render_area = area;
    }

    fn draw(&self, _params: Self::Params, frame: &mut Frame) {
        let mut area = self._render_area;
        if area.is_empty() {
            return;
        }

        let left = area.left();
        for ch in self
            .input
            .txt()
            .chars()
            .skip(self.scroll_offset)
            .take(area.width as usize)
        {
            area.x = frame
                .buffer_mut()
                .set_stringn(area.left(), area.top(), ch.to_string(), 1, Style::default())
                .0;
        }

        let cursor_in_range = self.input.cursor() - self.scroll_offset;
        frame.set_cursor_position(Position::new(left + cursor_in_range as u16, area.top()));
        let cursor_style = if self.mode.is_write_mode() {
            SetCursorStyle::BlinkingBar
        } else {
            SetCursorStyle::BlinkingBlock
        };

        if let Some((start, end)) = self.input.selection_range() {
            let start_left = start.saturating_sub(self.scroll_offset) as u16;
            let end_right = (end.min(self.scroll_offset + area.width as usize))
                .saturating_sub(self.scroll_offset) as u16;

            frame.buffer_mut().set_style(
                Rect {
                    x: left + start_left,
                    width: end_right.saturating_sub(start_left),
                    ..area
                },
                Style::default().red(),
            );
        }

        // if self.mode.is_write_mode() {
        //     // draw the suggestion
        //     if let Some(ref suggest) = self.suggestion {
        //         frame.buffer_mut().set_string(
        //             area.left(),
        //             area.top(),
        //             suggest,
        //             Style::default().dark_gray(),
        //         );
        //     }
        // }

        // TODO: is this hacky?
        let _ = execute!(stdout(), cursor_style);
    }
}

// pub struct RangeStyle {
//     range: (usize, usize),
//     style: Style,
// }

struct InputModeEditHandler {
    mode: InputMode,
    key: KeyEvent,
}

impl EditHandler for InputModeEditHandler {
    fn edit(&mut self, input: &mut Input) -> super::input::InputModified {
        let is_write_mode = self.mode.is_write_mode();
        let is_read_mode = self.mode.is_read_mode();

        match self.key.code {
            crossterm::event::KeyCode::Esc => {
                if is_write_mode {
                    self.mode = InputMode::Read;
                } else {
                    //  we need clear selection
                    input.clear_selection();
                }
            }
            KeyCode::Backspace | KeyCode::Delete if is_write_mode => input.delete_char(),
            KeyCode::Left => input.backward_cursor(1),
            KeyCode::Right => input.forward_cursor(1),
            KeyCode::Home => input.backward_cursor(input.cursor()),
            KeyCode::End => input.forward_cursor(input.txt().chars().count()),
            KeyCode::Char(ch) => match ch {
                'l' if !is_write_mode => input.forward_cursor(1),
                'h' if !is_write_mode => input.backward_cursor(1),
                'v' if !is_write_mode => {
                    if !input.has_range() {
                        input.start_selection();
                    } else {
                        input.clear_selection();
                    }
                }
                'd' if !is_write_mode => {
                    if input.has_range() {
                        input.delete_range();
                    } else {
                        input.delete_char();
                    }
                }
                'i' if is_read_mode => self.mode = InputMode::Write,
                'e' if is_read_mode => {
                    input.next_cursor_until(|_ch, next_ch| next_ch.map_or(true, |ch| ch == ' '))
                }
                'b' if is_read_mode => {
                    input.back_cursor_until(|prev_ch, _ch| prev_ch.map_or(true, |ch| ch == ' '))
                }
                _ => {
                    if is_write_mode {
                        input.insert_char(ch);
                    }
                }
            },
            _ => return false,
        }

        false
    }
}
