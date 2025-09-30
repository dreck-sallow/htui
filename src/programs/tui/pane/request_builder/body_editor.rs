use std::{io::Stdout, path::PathBuf};

use arboard::Clipboard;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    prelude::CrosstermBackend,
    style::{Style, Stylize},
    text::Span,
    Terminal,
};

use crate::{
    app_project::models::BodyContent,
    programs::tui::{
        common::{
            action_history::{ActionHistory, History, TrackAction},
            component::WithHistory,
            Interactive, UiComposedElement,
        },
        config::{keybinding, Config},
        elements::{dropdown::OverlayDropdown, utils::center_area},
        event_handler::{AppMessage, Events},
    },
};

use super::{body_binary::BodyBinaryEditor, body_form::BodyFormEditor, body_text::BodyTextEditor};

pub enum BodyEditorType {
    None { render_area: Rect },
    Binary(BodyBinaryEditor),
    Text(BodyTextEditor),
    Form(BodyFormEditor),
}

impl BodyEditorType {
    fn as_type(&self) -> BodyType {
        match self {
            BodyEditorType::None { .. } => BodyType::None,
            BodyEditorType::Binary(_) => BodyType::Binary,
            BodyEditorType::Text(_) => BodyType::Text,
            BodyEditorType::Form(_) => BodyType::Form,
        }
    }

    fn render_area(&self) -> Rect {
        match self {
            BodyEditorType::None { render_area } => *render_area,
            BodyEditorType::Binary(body_binary_editor) => body_binary_editor.render_area(),
            BodyEditorType::Text(body_text_editor) => body_text_editor.render_area(),
            BodyEditorType::Form(body_form_editor) => body_form_editor.render_area(),
        }
    }

    fn from_model(model: &BodyContent, content_area: Rect, viewport_area: Rect) -> BodyEditorType {
        match model {
            BodyContent::Empty => BodyEditorType::None {
                render_area: content_area,
            },
            BodyContent::File(path) => {
                let mut editor = BodyBinaryEditor::new();
                editor.set_area(content_area, viewport_area);
                editor.set_data(path.to_owned());
                BodyEditorType::Binary(editor)
            }
            BodyContent::Form(params) => {
                let mut editor = BodyFormEditor::new();
                editor.set_area(content_area, viewport_area);
                editor.set_data(params.to_owned());

                BodyEditorType::Form(editor)
            }
            BodyContent::Text(txt) => {
                let mut editor = BodyTextEditor::new();
                editor.set_area(content_area, viewport_area);
                editor.set_data(&txt);
                BodyEditorType::Text(editor)
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BodyType {
    None,
    Binary,
    Form,
    Text,
}

impl BodyType {
    pub fn into_empty_model(&self) -> BodyContent {
        match self {
            BodyType::None => BodyContent::Empty,
            BodyType::Binary => BodyContent::File(PathBuf::new()),
            BodyType::Form => BodyContent::Form(Vec::new()),
            BodyType::Text => BodyContent::Text(String::new()),
        }
    }
}

impl AsRef<str> for BodyType {
    fn as_ref(&self) -> &str {
        match self {
            BodyType::None => "None",
            BodyType::Form => "Form",
            BodyType::Text => "Text",
            BodyType::Binary => "Binary",
        }
    }
}

pub struct BodyEditor {
    editor: BodyEditorType,
    dropdown: OverlayDropdown<BodyType>,
    show_dropdown: bool,
    dropdown_area: Rect,
    content_area: Rect,
    _history: ActionHistory<BodyEditorAction>,
}

impl BodyEditor {
    pub fn new(config: &Config) -> Self {
        Self {
            editor: BodyEditorType::None {
                render_area: Rect::default(),
            },
            dropdown: OverlayDropdown::with_items(
                BodyType::None,
                [
                    BodyType::None,
                    BodyType::Text,
                    BodyType::Binary,
                    BodyType::Form,
                ],
            )
            .with_style(
                Style::default()
                    .fg(config.theme.dropdown.fg)
                    .bg(config.theme.dropdown.bg),
            )
            .with_highlight_style(
                Style::default()
                    .fg(config.theme.dropdown_highlight.fg)
                    .bg(config.theme.dropdown_highlight.bg),
            ),
            show_dropdown: false,
            dropdown_area: Rect::default(),
            content_area: Rect::default(),
            _history: ActionHistory::new(),
        }
    }

    pub fn set_state(&mut self, body_content: BodyContent) {
        self._history.apply(
            BodyEditorAction::SetBody(body_content.clone()),
            &mut self.editor,
        );
    }

    pub fn get_data(&self) -> BodyContent {
        match &self.editor {
            BodyEditorType::None { .. } => BodyContent::Empty,
            BodyEditorType::Binary(body_binary_editor) => body_binary_editor.body_model(),
            BodyEditorType::Text(body_text_editor) => body_text_editor.body_model(),
            BodyEditorType::Form(body_form_editor) => body_form_editor.body_model(),
        }
    }
}

impl<'params> UiComposedElement<'params> for BodyEditor {
    type Params = &'params Config;

    fn set_area(&mut self, area: Rect, viewport_area: Rect) {
        let [dropdown_area, content_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);

        self.dropdown_area = dropdown_area;
        self.content_area = content_area;

        match &mut self.editor {
            BodyEditorType::None { .. } => {
                self.editor = BodyEditorType::None {
                    render_area: content_area,
                }
            }
            BodyEditorType::Binary(body_binary_editor) => {
                body_binary_editor.set_area(content_area, viewport_area)
            }
            BodyEditorType::Text(body_text_editor) => {
                body_text_editor.set_area(content_area, viewport_area)
            }
            BodyEditorType::Form(body_form_editor) => {
                body_form_editor.set_area(content_area, viewport_area)
            }
        }
    }

    fn draw(&self, config: Self::Params, frame: &mut ratatui::Frame) {
        let title = Span::from(format!(" Type: {} ", self.editor.as_type().as_ref())).italic();
        let body_type = Span::from("\u{25bc} ");

        let [title_area, body_type_area] = Layout::horizontal([
            Constraint::Length(title.width() as u16),
            Constraint::Length(body_type.width() as u16),
        ])
        .flex(ratatui::layout::Flex::SpaceBetween)
        .areas(self.dropdown_area);

        frame.render_widget(title, title_area);
        frame.render_widget(body_type, body_type_area);
        frame.buffer_mut().set_style(
            self.dropdown_area,
            Style::default()
                .fg(config.theme.dropdown.fg)
                .bg(config.theme.dropdown.bg),
        );

        match &self.editor {
            BodyEditorType::None { .. } => {}
            BodyEditorType::Binary(body_binary_editor) => body_binary_editor.draw(config, frame),
            BodyEditorType::Text(body_text_editor) => body_text_editor.draw((), frame),
            BodyEditorType::Form(body_form_editor) => body_form_editor.draw(config, frame),
        }
    }

    fn draw_overlay(&self, params: Self::Params, frame: &mut ratatui::Frame) {
        if self.show_dropdown {
            let area = Rect {
                y: self.dropdown_area.top() + 1,
                height: 4,
                ..self.dropdown_area
            };

            frame.render_widget(&self.dropdown, area);
        } else {
            match &self.editor {
                BodyEditorType::None { .. } => {
                    let area = center_area(
                        self.content_area,
                        Constraint::Length(1),
                        Constraint::Length(7),
                    );
                    frame.render_widget("No body", area);
                }
                BodyEditorType::Binary(body_binary_editor) => {
                    body_binary_editor.draw_overlay(params, frame);
                }
                BodyEditorType::Text(_) => {}
                BodyEditorType::Form(body_form_editor) => {
                    body_form_editor.draw_overlay(params, frame);
                }
            }
        }
    }
}

impl<'params> Interactive<'params> for BodyEditor {
    type Effect = ();

    type Params = (
        &'params Config,
        &'params mut Events<AppMessage>,
        &'params mut Terminal<CrosstermBackend<Stdout>>,
        &'params mut Clipboard,
    );

    fn is_input_focus(&self) -> bool {
        match &self.editor {
            BodyEditorType::None { .. } => false,
            BodyEditorType::Binary(body_binary_editor) => body_binary_editor.is_input_focus(),
            BodyEditorType::Text(body_text_editor) => body_text_editor.is_input_focus(),
            BodyEditorType::Form(body_form_editor) => body_form_editor.is_input_focus(),
        }
    }

    fn is_visible_overlay(&self) -> bool {
        if self.show_dropdown {
            return true;
        }

        match &self.editor {
            BodyEditorType::None { .. } => false,
            BodyEditorType::Binary(body_binary_editor) => body_binary_editor.is_visible_overlay(),
            BodyEditorType::Text(body_text_editor) => body_text_editor.is_visible_overlay(),
            BodyEditorType::Form(body_form_editor) => body_form_editor.is_visible_overlay(),
        }
    }

    fn handle_key(
        &mut self,
        (config, events, terminal, clipboard): Self::Params,
        key: crossterm::event::KeyEvent,
    ) -> Self::Effect {
        if self.show_dropdown {
            if let Some(key) = config.keymap.match_global_action(key) {
                match key {
                    keybinding::GlobalKeyAction::MoveDown => {
                        self.dropdown.next();
                    }
                    keybinding::GlobalKeyAction::MoveUp => {
                        self.dropdown.prev();
                    }
                    keybinding::GlobalKeyAction::ClosePopup => {
                        self.show_dropdown = false;
                    }
                    keybinding::GlobalKeyAction::SubmitPopup => {
                        self.show_dropdown = false;

                        self._history.apply(
                            BodyEditorAction::SetBody(self.dropdown.selected().into_empty_model()),
                            &mut self.editor,
                        );
                    }
                    _ => {}
                }
            }
        } else {
            if !self.is_visible_overlay() {
                if let Some(action) = config.keymap.match_request_builder_action(key) {
                    match action {
                        keybinding::RequestBuilderKeyAction::OpenDropdown => {
                            self.show_dropdown = true;
                            return;
                        }
                        _ => {}
                    }
                }
            }

            match &mut self.editor {
                BodyEditorType::None { .. } => {}
                BodyEditorType::Binary(body_binary_editor) => {
                    body_binary_editor.handle_key(config, key)
                }
                BodyEditorType::Text(body_text_editor) => {
                    body_text_editor.handle_key((config, events, terminal), key)
                }
                BodyEditorType::Form(body_form_editor) => {
                    body_form_editor.handle_key((config, clipboard), key)
                }
            }
        }
    }
}

enum BodyEditorAction {
    SetBody(BodyContent),
}

impl TrackAction for BodyEditorAction {
    type State = BodyEditorType;

    fn apply(&self, state: &mut Self::State) -> Option<Self>
    where
        Self: Sized,
    {
        match self {
            BodyEditorAction::SetBody(body_content) => {
                let previous_body_content = match state {
                    BodyEditorType::None { .. } => BodyContent::Empty,
                    BodyEditorType::Binary(body_binary_editor) => body_binary_editor.body_model(),
                    BodyEditorType::Text(body_text_editor) => body_text_editor.body_model(),
                    BodyEditorType::Form(body_form_editor) => body_form_editor.body_model(),
                };

                *state =
                    BodyEditorType::from_model(body_content, state.render_area(), Rect::default());

                Some(Self::SetBody(previous_body_content))
            }
        }
    }
}

impl WithHistory for BodyEditor {
    fn undo(&mut self) {
        self._history.undo(&mut self.editor);
    }

    fn redo(&mut self) {
        self._history.redo(&mut self.editor);
    }
}
