use std::io::stdout;

use crossterm::{
    cursor::SetCursorStyle,
    event::{KeyCode, KeyEvent},
    execute,
};
use ratatui::{
    layout::{Position, Rect},
    style::{Style, Stylize},
    widgets::Widget,
    Frame,
};

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
}

impl ModeInput {
    pub fn new() -> Self {
        Self {
            mode: InputMode::default(),
            input: Input::default(),
            // suggestion: None,
        }
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
    }
}

impl ModeInput {
    pub fn draw(&self, mut area: ratatui::prelude::Rect, frame: &mut Frame) {
        if area.is_empty() {
            return;
        }

        let left = area.left();

        for ch in self.input.txt().chars() {
            area.x = frame
                .buffer_mut()
                .set_stringn(area.left(), area.top(), ch.to_string(), 1, Style::default())
                .0;
        }

        frame.set_cursor_position(Position::new(left + self.input.cursor() as u16, area.top()));
        let cursor_style = if self.mode.is_write_mode() {
            SetCursorStyle::BlinkingBar
        } else {
            SetCursorStyle::BlinkingBlock
        };

        if let Some((start, end)) = self.input.selection_range() {
            frame.buffer_mut().set_style(
                Rect {
                    x: left + start as u16,
                    width: (end - start) as u16 + 1,
                    ..area
                },
                Style::default().red(),
            );
        }

        // TODO: is this hacky?
        let _ = execute!(stdout(), cursor_style);
    }
}

impl Widget for &ModeInput {
    fn render(self, mut area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        if area.is_empty() {
            return;
        }

        // let left = area.left();

        for ch in self.input.txt().chars() {
            area.x = buf
                .set_stringn(area.left(), area.top(), ch.to_string(), 1, Style::default())
                .0;
        }

        // if let Some(cell) = buf.cell_mut((left + self.input.cursor() as u16, area.top())) {
        //     let (cursor, cursor_style) = if self.mode.is_write_mode() {
        //         (symbols::block::ONE_EIGHTH, Style::default().red())
        //     } else {
        //         (" ", Style::default().reversed())
        //     };
        //     cell.set_symbol(cursor).set_style(cursor_style);
        // }

        // let _ = stdout().execute(Show).execute();
        // println!("is_error: {}", e.is_err());
    }
}

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
            KeyCode::End => input.forward_cursor(usize::MAX),
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
                // 'v' if is_read_mode => self.mode = InputMode::Read,
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
