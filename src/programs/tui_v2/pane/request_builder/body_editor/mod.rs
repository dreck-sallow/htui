use crossterm::event::{KeyEvent, KeyModifiers};
use form_editor::{FormDataEditor, FormUrlEncodedEditor};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    text::Span,
    widgets::Clear,
    Frame,
};
use text_editor::TextEditor;

use crate::programs::tui_v2::{
    common::{
        elements::{ui_block, ui_placeholder, utils::center_area},
        select_list::SelectList,
    },
    pane::state::{BodyContent, PaneState},
};

mod file_editor;
mod form_editor;
mod text_editor;

enum TypeEditor {
    None(NoneEditor),
    Text(TextEditor),
    FormData(FormDataEditor),
    FormUrlEncoded(FormUrlEncodedEditor),
    File,
}

impl TypeEditor {
    pub fn none() -> Self {
        Self::None(NoneEditor)
    }

    #[inline]
    pub fn as_plain(&self) -> PlainBodyType {
        match self {
            TypeEditor::None(_) => PlainBodyType::None,
            TypeEditor::Text(_) => PlainBodyType::Text,
            TypeEditor::FormData(_) => PlainBodyType::FormData,
            TypeEditor::FormUrlEncoded(_) => PlainBodyType::FormUrlEncoded,
            TypeEditor::File => PlainBodyType::File,
        }
    }

    pub fn from_plain(plain: PlainBodyType) -> Self {
        match plain {
            PlainBodyType::None => TypeEditor::none(),
            PlainBodyType::Text => TypeEditor::Text(TextEditor::new()),
            PlainBodyType::FormData => TypeEditor::FormData(FormDataEditor::default()),
            PlainBodyType::FormUrlEncoded => {
                TypeEditor::FormUrlEncoded(FormUrlEncodedEditor::default())
            }
            PlainBodyType::File => TypeEditor::File,
        }
    }

    pub fn from_state(body: &BodyContent) -> Self {
        match body {
            BodyContent::None => TypeEditor::none(),
            BodyContent::File(_) => TypeEditor::none(),
            BodyContent::FormUrlEncoded(params) => {
                TypeEditor::FormUrlEncoded(FormUrlEncodedEditor::new(params.clone()))
            }
            BodyContent::FormData(data) => TypeEditor::FormData(FormDataEditor::new(data.clone())),
            BodyContent::Text(st) => {
                let mut editor = TextEditor::new();
                editor.set_content(st);
                TypeEditor::Text(editor)
            }
        }
    }
}

const BODY_LIST: [PlainBodyType; 5] = [
    PlainBodyType::None,
    PlainBodyType::Text,
    PlainBodyType::FormData,
    PlainBodyType::FormUrlEncoded,
    PlainBodyType::File,
];

pub struct BodyEditor {
    editor: TypeEditor,
    select: SelectList<PlainBodyType>,
    show_select: bool,
}

impl BodyEditor {
    pub fn new() -> Self {
        let mut select = SelectList::new(&BODY_LIST);
        select.select(&PlainBodyType::None);

        Self {
            editor: TypeEditor::None(NoneEditor),
            select,
            show_select: false,
        }
    }

    pub fn sync(&mut self, state: &mut PaneState) {
        if let Some(req) = state.collections.current_req_mut() {
            self.editor = TypeEditor::from_state(&req.body);
            self.select.select(&PlainBodyType::from_model(&req.body));
        }
    }
}

impl BodyEditor {
    pub fn draw(&self, area: Rect, frame: &mut Frame) {
        let [editor_area, type_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

        match &self.editor {
            TypeEditor::None(e) => {
                e.draw(editor_area, frame);
            }
            TypeEditor::Text(e) => {
                e.draw(editor_area, frame);
            }
            TypeEditor::FormData(e) => {
                e.draw(editor_area, frame);
            }
            TypeEditor::FormUrlEncoded(e) => {
                e.draw(editor_area, frame);
            }
            TypeEditor::File => {}
        }

        let label = Span::raw(format!(" {} ", self.editor.as_plain().to_string()))
            .on_light_blue()
            .dark_gray();
        frame.render_widget(label, type_area);
    }

    pub fn draw_overlay(&self, frame: &mut Frame) {
        if self.show_select {
            let area = center_area(
                frame.area(),
                Constraint::Length(BODY_LIST.len() as u16 + 2),
                Constraint::Percentage(40),
            );
            let block = ui_block(" Select body type", true);
            let inner_area = block.inner(area);

            frame.render_widget(Clear, area);
            frame.render_widget(block, area);
            self.select.draw(inner_area, frame);
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent, body: &mut BodyContent) {
        if self.show_select {
            match key.code {
                crossterm::event::KeyCode::Enter => {
                    let selected = self.select.selected().unwrap();

                    if *selected == PlainBodyType::from_model(&body) {
                        self.editor = TypeEditor::from_state(&body)
                    } else {
                        self.editor = TypeEditor::from_plain(*selected);
                    }
                    self.show_select = false;
                }
                crossterm::event::KeyCode::Esc => {
                    self.select.select(&self.editor.as_plain());
                    self.show_select = false;
                }
                _ => {
                    self.select.handle_key(key);
                }
            }
        } else {
            match key.code {
                crossterm::event::KeyCode::Char('N') if key.modifiers == KeyModifiers::SHIFT => {
                    self.show_select = true;
                }
                _ => match &mut self.editor {
                    TypeEditor::None(_) => {}
                    TypeEditor::Text(e) => e.handle_key(key),
                    TypeEditor::FormData(e) => {}
                    TypeEditor::FormUrlEncoded(e) => {}
                    TypeEditor::File => {}
                },
            }
        }
    }
}

pub struct NoneEditor;

impl NoneEditor {
    pub fn draw(&self, area: Rect, frame: &mut Frame) {
        ui_placeholder("No Body").draw(area, frame);
    }
}

#[derive(PartialEq, Clone, Copy)]
enum PlainBodyType {
    None,
    Text,
    FormData,
    FormUrlEncoded,
    File,
}

impl PlainBodyType {
    pub fn from_model(body: &BodyContent) -> Self {
        match body {
            BodyContent::None => Self::None,
            BodyContent::File(_) => Self::File,
            BodyContent::FormData(_) => Self::FormData,
            BodyContent::FormUrlEncoded(_) => Self::FormUrlEncoded,
            BodyContent::Text(_) => Self::Text,
        }
    }
}

impl ToString for PlainBodyType {
    fn to_string(&self) -> String {
        let raw = match self {
            Self::None => "None",
            Self::Text => "Text",
            Self::FormData => "Form (multipart)",
            Self::FormUrlEncoded => "Form (url-encoded)",
            Self::File => "File",
        };

        raw.to_string()
    }
}
