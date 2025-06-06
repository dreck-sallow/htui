use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    style::{Style, Stylize},
    text::Span,
    widgets::Widget,
};
use tui_textarea::{CursorMove, Input, TextArea};

pub enum EditMode {
    Visual,
    Select,
    Insert,
}

impl EditMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            EditMode::Visual => " Visual ",
            EditMode::Select => " Select ",
            EditMode::Insert => " Insert ",
        }
    }
}

// impl Styled for EditMode {
//     type Item = Span<'static>;

//     fn style(&self) -> Style {
//         match self {
//             EditMode::Visual => Style::default().green(),
//             EditMode::Select => Style::default().yellow(),
//             EditMode::Insert => Style::default().magenta(),
//         }
//     }

//     fn set_style<S: Into<Style>>(self, style: S) -> Self::Item {
//         let txt = match self {
//             EditMode::Visual => " Visual ",
//             EditMode::Select => " Select ",
//             EditMode::Insert => " Insert ",
//         };

//         Span::styled(txt, style)
//     }
// }

pub struct TextEditor {
    textarea: TextArea<'static>,
    mode: EditMode,
}

impl TextEditor {
    pub fn new() -> Self {
        let mut textarea = TextArea::default();
        textarea.set_line_number_style(Style::default().on_dark_gray());
        textarea.set_cursor_line_style(Style::default().on_dark_gray());

        Self {
            textarea,
            mode: EditMode::Visual,
        }
    }

    pub fn set_mode(&mut self, mode: EditMode) {
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
        // let input = Input::from(key);
        // self.textarea.input(input);

        if let KeyEventKind::Press = key.kind {
            let new_mode = match key.code {
                KeyCode::Esc => Some(EditMode::Visual),
                KeyCode::Char('v') => {
                    if let EditMode::Visual = self.mode {
                        Some(EditMode::Select)
                    } else {
                        None
                    }
                }
                KeyCode::Char('i') => {
                    if let EditMode::Insert = self.mode {
                        None
                    } else {
                        Some(EditMode::Insert)
                    }
                }
                _ => None,
            };

            match new_mode {
                Some(mode) => self.set_mode(mode),
                None => match key.code {
                    key_code => match self.mode {
                        EditMode::Visual => match key_code {
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
                            _ => {}
                        },
                        EditMode::Select => match key_code {
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
                            _ => {}
                        },
                        EditMode::Insert => {
                            let input = Input::from(key);
                            self.textarea.input(input);
                        }
                    },
                },
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

        let mode = Span::from(self.mode.as_str());
        area.y = editor_area.bottom();
        area.height = 1;

        buf.set_style(area, Style::default().on_dark_gray());
        buf.set_span(area.x, area.y, &mode, 100);
    }
}
