use std::{collections::HashMap, path::PathBuf};

use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::Span,
};

use crate::{
    programs::tui::{
        element_view::{Drawable, Interactive},
        elements::dropdown::OverlayDropdown_v2,
    },
    store::models::BodyContent,
};

use super::{
    editor::TextEditor,
    pane_state::{history::MutationCollector, PaneState},
};

/// Body content type
/// mirror from `BodyContent`
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BodyType {
    Empty,
    File,
    Form,
    Text,
}

impl AsRef<str> for BodyType {
    fn as_ref(&self) -> &str {
        match self {
            BodyType::Empty => "Empty",
            BodyType::File => "File",
            BodyType::Form => "Form",
            BodyType::Text => "Text",
        }
    }
}

impl From<&BodyContent> for BodyType {
    fn from(value: &BodyContent) -> Self {
        match value {
            BodyContent::Empty => BodyType::Empty,
            BodyContent::File(_) => BodyType::File,
            BodyContent::Form(_) => BodyType::Form,
            BodyContent::Text(_) => BodyType::Text,
        }
    }
}

pub struct BodyEditorComponent {
    body_content: BodyContent,
    editable: bool,
    render_area: Rect,
    text_editor: TextEditor,
    dropdown: OverlayDropdown_v2<BodyType>,
    show_dropdown: bool,
}

impl BodyEditorComponent {
    pub fn new(editable: bool) -> Self {
        Self {
            render_area: Rect::default(),
            editable,
            body_content: BodyContent::Empty,
            text_editor: TextEditor::new(editable),
            dropdown: OverlayDropdown_v2::with_items(
                BodyType::Empty,
                [
                    BodyType::Empty,
                    BodyType::File,
                    BodyType::Form,
                    BodyType::Text,
                ],
            )
            .with_highlight_style(Style::default().blue()),
            show_dropdown: false,
        }
    }

    pub fn is_editing(&self) -> bool {
        if self.editable {
            return match self.body_content {
                BodyContent::Empty => false,
                BodyContent::File(_) => false,
                _ => self.text_editor.mode().is_write_mode(),
            };
        }
        false
    }

    // pub fn body_type(&self) -> &BodyContent {
    //     &self.body_content
    // }

    pub fn draw_header_line(&self, line_area: Rect, frame: &mut ratatui::Frame) {
        let title = Span::from(format!(" Type: {} ", self.body_content.as_tag())).italic();
        let body_type = Span::from("\u{25bc} ");

        let [title_area, body_type_area] = Layout::horizontal([
            Constraint::Length(title.width() as u16),
            Constraint::Length(body_type.width() as u16),
        ])
        .flex(ratatui::layout::Flex::SpaceBetween)
        .areas(line_area);

        frame.render_widget(title, title_area);
        frame.render_widget(body_type, body_type_area);
        frame
            .buffer_mut()
            .set_style(line_area, Style::default().on_dark_gray());
    }
}

impl<'a: 'painter, 'painter> Drawable<'a, 'painter> for BodyEditorComponent {
    type State = PaneState;

    fn draw(
        &'a self,
        painter: &mut crate::programs::tui::element_view::Painter<'painter>,
        _state: &'a Self::State,
    ) {
        painter.render(|frame| {
            let content_area = {
                if self.editable {
                    let [header_area, content_area] =
                        Layout::vertical([Constraint::Length(1), Constraint::Fill(1)])
                            .areas(self.render_area);

                    self.draw_header_line(header_area, frame);
                    content_area
                } else {
                    self.render_area
                }
            };

            match &self.body_content {
                BodyContent::Empty => {
                    let text = Span::from("No body").italic();
                    let [inner_area] = Layout::vertical([Constraint::Length(1)])
                        .flex(ratatui::layout::Flex::Center)
                        .areas(content_area);
                    let [inner_area] =
                        Layout::horizontal([Constraint::Length(text.width() as u16)])
                            .flex(ratatui::layout::Flex::Center)
                            .areas(inner_area);

                    frame.render_widget(text, inner_area);
                }
                BodyContent::File(path_buf) => {
                    let text = Span::from(path_buf.as_os_str().to_str().unwrap()).italic();
                    let [inner_area] = Layout::vertical([Constraint::Length(1)])
                        .flex(ratatui::layout::Flex::Center)
                        .areas(content_area);
                    let [inner_area] =
                        Layout::horizontal([Constraint::Length(text.width() as u16)])
                            .flex(ratatui::layout::Flex::Center)
                            .areas(inner_area);

                    frame.render_widget(text, inner_area);
                }
                BodyContent::Form(_hash_map) => {
                    // frame.render_widget(Span::from("Form values!"), content_area);
                    frame.render_widget(&self.text_editor, content_area);
                }
                BodyContent::Text(_) => {
                    frame.render_widget(&self.text_editor, content_area);
                }
            }
        });

        if self.show_dropdown {
            painter.render_last(|frame| {
                let area = Rect {
                    y: self.render_area.top() + 1,
                    height: 5,
                    ..self.render_area
                };

                frame.render_widget(&self.dropdown, area);
            });
        }
    }

    fn set_render_area(&mut self, _area: Rect) {
        self.render_area = _area;
    }
}

impl<'a: 'painter, 'painter> Interactive<'a, 'painter> for BodyEditorComponent {
    type Mutator = MutationCollector<'a>;

    fn on_key(
        &mut self,
        key: crossterm::event::KeyEvent,
        _mutator: &mut Self::Mutator,
        _state: &Self::State,
    ) {
        if self.show_dropdown {
            match key.code {
                KeyCode::Enter => {
                    self.show_dropdown = false;
                    self.body_content = match self.dropdown.selected() {
                        BodyType::Empty => BodyContent::Empty,
                        BodyType::File => BodyContent::File(PathBuf::new()),
                        BodyType::Form => BodyContent::Form(HashMap::new()),
                        BodyType::Text => BodyContent::Text(String::new()),
                    };
                }
                KeyCode::Esc => {
                    self.show_dropdown = false;
                }
                _ => {
                    self.dropdown.handle_key(key);
                }
            }
        } else {
            match key.code {
                KeyCode::Enter if key.modifiers == KeyModifiers::ALT => {
                    self.show_dropdown = true;
                    self.dropdown.select(BodyType::from(&self.body_content));
                }
                _ => match &self.body_content {
                    BodyContent::Empty => {}
                    BodyContent::File(_) => {
                        //TODO: Show a menu for select files
                    }
                    _ => {
                        self.text_editor.handle_key(key);
                    }
                },
            }
        }
    }

    fn on_change_state(&mut self, state: &Self::State) {
        let reader = state.reader();

        if let Some(req) = reader.current_request() {
            self.body_content = req.body().clone();

            match &self.body_content {
                BodyContent::Empty => {}
                BodyContent::File(_) => {}
                BodyContent::Form(map) => {
                    self.text_editor.clean_lines();
                    for (k, v) in map {
                        self.text_editor.insert_str(&format!("{k}: {v}"));
                    }
                }
                BodyContent::Text(txt) => {
                    self.text_editor.clean_lines();
                    self.text_editor.insert_str(txt);
                }
            }
        }
    }
}
