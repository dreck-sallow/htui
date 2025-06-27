use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    style::{Style, Stylize},
    text::Span,
    widgets::Widget,
};
use tui_textarea::{CursorMove, Input, TextArea};

#[derive(Clone, Copy)]
pub enum EditMode {
    Write,
    Read,
    ReadOnly,
}

impl EditMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            EditMode::Write => " Write ",
            EditMode::Read => " Read ",
            EditMode::ReadOnly => " Read-only ",
        }
    }

    pub fn is_read_mode(&self) -> bool {
        matches!(self, EditMode::Read)
    }

    pub fn is_write_mode(&self) -> bool {
        matches!(self, EditMode::Write)
    }

    pub fn is_read_only_mode(&self) -> bool {
        matches!(self, EditMode::ReadOnly)
    }
}

pub struct TextEditor {
    textarea: TextArea<'static>,
    mode: EditMode,
}

impl TextEditor {
    pub fn new(editable: bool) -> Self {
        let mut textarea = TextArea::default();
        textarea.set_line_number_style(Style::default().on_dark_gray());
        textarea.set_cursor_line_style(Style::default());

        Self {
            textarea,
            mode: if editable {
                EditMode::Read
            } else {
                EditMode::ReadOnly
            },
        }
    }

    pub fn mode(&self) -> EditMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: EditMode) {
        // TODO: Set the cursor style
        self.mode = mode;
    }

    pub fn clean_lines(&mut self) {
        let lines_count = self.textarea.lines().len();
        self.textarea.move_cursor(CursorMove::Bottom);

        for _i in [lines_count..0] {
            self.textarea.delete_line_by_head();
            self.textarea.delete_newline();
        }
    }

    pub fn insert_str(&mut self, txt: &str) {
        self.textarea.insert_str(txt);
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if let KeyEventKind::Press = key.kind {
            let new_mode = match key.code {
                KeyCode::Esc => {
                    if self.mode.is_read_only_mode() {
                        None
                    } else {
                        Some(EditMode::Read)
                    }
                }
                KeyCode::Char('i') => {
                    if self.mode.is_read_mode() {
                        Some(EditMode::Write)
                    } else {
                        None
                    }
                }
                _ => None,
            };

            match new_mode {
                Some(mode) => self.set_mode(mode),
                None => {
                    if self.mode.is_write_mode() {
                        let input = Input::from(key);
                        self.textarea.input(input);
                    } else {
                        match key.code {
                            KeyCode::Char('j') | KeyCode::Down => {
                                self.textarea.move_cursor(CursorMove::Down)
                            }
                            KeyCode::Char('k') | KeyCode::Up => {
                                self.textarea.move_cursor(CursorMove::Up)
                            }
                            KeyCode::Char('l') | KeyCode::Right => {
                                self.textarea.move_cursor(CursorMove::Forward)
                            }
                            KeyCode::Char('h') | KeyCode::Left => {
                                self.textarea.move_cursor(CursorMove::Back)
                            }
                            KeyCode::Char('e') => {
                                self.textarea.move_cursor(CursorMove::WordForward)
                            }
                            KeyCode::Char('b') => self.textarea.move_cursor(CursorMove::WordBack),
                            KeyCode::Char('d') => {
                                self.textarea.delete_char();
                            }
                            KeyCode::Char('x') => {
                                self.textarea.delete_line_by_end();
                                self.textarea.delete_line_by_head();
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}

impl Widget for &TextEditor {
    fn render(self, mut area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let mut editor_area = area.clone();
        editor_area.height = editor_area.height.saturating_sub(1);

        self.textarea.render(editor_area, buf);

        let mode_style = match self.mode {
            EditMode::Read => Style::default().on_yellow().black(),
            EditMode::ReadOnly => Style::default().on_red().black(),
            EditMode::Write => Style::default().on_blue().black(),
        };

        let mode = Span::from(self.mode.as_str()).style(mode_style);
        area.y = editor_area.bottom();
        area.height = 1;

        buf.set_style(area, Style::default().on_dark_gray());
        buf.set_span(area.x, area.y, &mode, 100);
    }
}
