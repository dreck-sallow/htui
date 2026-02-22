use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    style::{Style, Stylize},
    text::Span,
    widgets::Widget,
};
use tui_textarea::{CursorMove, Input, TextArea};

use super::input::input_mode::Mode;

pub struct TextEditor {
    textarea: TextArea<'static>,
    mode: Mode,
}

impl TextEditor {
    pub fn new(editable: bool) -> Self {
        let mut textarea = TextArea::default();
        textarea.set_line_number_style(Style::default().on_dark_gray());
        textarea.set_cursor_line_style(Style::default());

        Self {
            textarea,
            mode: if editable {
                Mode::Normal
            } else {
                Mode::Readonly
            },
        }
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: Mode) {
        // TODO: Set the cursor style
        self.mode = mode;
    }

    pub fn lines(&self) -> &[String] {
        self.textarea.lines()
    }

    pub fn clean_lines(&mut self) {
        let lines_count = self.textarea.lines().len();
        self.textarea.move_cursor(CursorMove::Bottom);
        self.textarea.move_cursor(CursorMove::Forward);

        for _i in 0..lines_count {
            self.textarea.delete_line_by_head();
            self.textarea.delete_newline();
        }
    }

    pub fn insert_str(&mut self, txt: &str) {
        self.textarea.insert_str(txt);
        self.textarea.move_cursor(CursorMove::Head);
    }

    // pub fn open_in_editor(&mut self) {
    //     let text = self.textarea.lines().join("\n");
    //     let mut file = NamedTempFile::new().unwrap();
    //     file.write_all(text.as_bytes()).unwrap();

    //     let file_path = file.into_temp_path();

    //     Command::new(std::env::var("EDITOR").unwrap())
    //         .args([&file_path])
    //         .status()
    //         .expect("Error executing editor");

    //     if self.mode.is_read_mode() {
    //         self.clean_lines();
    //         self.insert_str(&fs::read_to_string(file_path).unwrap());
    //     }
    // }

    pub fn handle_key(&mut self, key: KeyEvent) {
        let new_mode = match key.code {
            KeyCode::Esc => self.mode.is_insert().then_some(Mode::Normal),
            KeyCode::Char('i') => self.mode.is_normal().then_some(Mode::Insert),
            _ => None,
        };

        match new_mode {
            Some(mode) => self.set_mode(mode),
            None => {
                if self.mode.is_insert() {
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
                        KeyCode::Char('u') if self.mode.is_normal() => {
                            if key.modifiers == KeyModifiers::SHIFT {
                                self.textarea.redo();
                            } else {
                                self.textarea.undo();
                            }
                        }
                        KeyCode::Char('e') => self.textarea.move_cursor(CursorMove::WordForward),
                        KeyCode::Char('b') => self.textarea.move_cursor(CursorMove::WordBack),
                        KeyCode::Char('d') if self.mode.is_normal() => {
                            self.textarea.delete_char();
                        }
                        KeyCode::Char('x') if self.mode.is_normal() => {
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

impl Widget for &TextEditor {
    fn render(self, mut area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let mut editor_area = area;
        editor_area.height = editor_area.height.saturating_sub(1);

        self.textarea.render(editor_area, buf);

        let mode_style = match self.mode {
            Mode::Normal => Style::default().on_yellow().black(),
            Mode::Readonly => Style::default().on_red().black(),
            Mode::Insert => Style::default().on_blue().black(),
        };

        let mode = Span::from(self.mode.as_str()).style(mode_style);
        area.y = editor_area.bottom();
        area.height = 1;

        buf.set_style(area, Style::default().on_dark_gray());
        buf.set_span(area.x, area.y, &mode, 100);
    }
}
