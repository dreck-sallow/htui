use std::{
    collections::HashMap,
    io::{stdout, Stdout},
};

use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
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

use super::text_editor::TextEditor;

#[derive(Clone, Copy, PartialEq, Eq)]
enum BodyTypeV2 {
    Empty,
    FormUrlEncoded,
    Text,
}

impl AsRef<str> for BodyTypeV2 {
    fn as_ref(&self) -> &str {
        match self {
            BodyTypeV2::Empty => "Empty",
            BodyTypeV2::FormUrlEncoded => "FormUrlEncoded",
            BodyTypeV2::Text => "Text",
        }
    }
}

pub struct BodyEditorComponent {
    body_content: BodyContent,
    render_area: Rect,
    text_editor: TextEditor,
    dropdown: OverlayDropdown<BodyTypeV2>,
    show_dropdown: bool,
    _history: ActionHistory<BodyEditorAction>,
}

impl BodyEditorComponent {
    pub fn new(config: &Config) -> Self {
        Self {
            body_content: BodyContent::Empty,
            render_area: Rect::default(),
            text_editor: TextEditor::new(true),
            dropdown: OverlayDropdown::with_items(
                BodyTypeV2::Empty,
                [
                    BodyTypeV2::Empty,
                    BodyTypeV2::FormUrlEncoded,
                    BodyTypeV2::Text,
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
            _history: ActionHistory::new(),
        }
    }

    pub fn is_editing(&self) -> bool {
        match self.body_content {
            BodyContent::Empty => false,
            BodyContent::File(_) => false,
            _ => self.text_editor.mode().is_write_mode(),
        }
    }

    pub fn set_state(&mut self, body_content: BodyContent) {
        self._history.clean();
        self.body_content = body_content;
        match &self.body_content {
            BodyContent::Empty => {}
            BodyContent::File(_) => {}
            BodyContent::Form(_) => {}
            BodyContent::Text(txt) => {
                self.text_editor.clean_lines();
                self.text_editor.insert_str(txt);
            }
        }
    }

    pub fn get_data(&self) -> BodyContent {
        match &self.body_content {
            BodyContent::Empty => BodyContent::Empty,
            BodyContent::File(path_buf) => BodyContent::File(path_buf.clone()),
            BodyContent::Form(hash_map) => BodyContent::Form(hash_map.clone()),
            BodyContent::Text(_) => BodyContent::Text(self.text_editor.lines().join("\n")),
        }
    }
}

impl<'params> UiComposedElement<'params> for BodyEditorComponent {
    type Params = &'params Config;

    fn set_area(&mut self, area: Rect, _viewport_area: Rect) {
        self.render_area = area;
    }

    fn draw(&self, config: Self::Params, frame: &mut ratatui::Frame) {
        let [header_area, content_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(self.render_area);

        let title = Span::from(format!(" Type: {} ", self.body_content.as_tag())).italic();
        let body_type = Span::from("\u{25bc} ");

        let [title_area, body_type_area] = Layout::horizontal([
            Constraint::Length(title.width() as u16),
            Constraint::Length(body_type.width() as u16),
        ])
        .flex(ratatui::layout::Flex::SpaceBetween)
        .areas(header_area);

        frame.render_widget(title, title_area);
        frame.render_widget(body_type, body_type_area);
        frame.buffer_mut().set_style(
            header_area,
            Style::default()
                .fg(config.theme.dropdown.fg)
                .bg(config.theme.dropdown.bg),
        );

        match &self.body_content {
            BodyContent::Empty => {
                let text = Span::from("No body").italic();
                let inner_area = center_area(
                    content_area,
                    Constraint::Length(1),
                    Constraint::Length(text.width() as u16),
                );

                frame.render_widget(text, inner_area);
            }
            BodyContent::File(path_buf) => {
                let text = Span::from(path_buf.as_os_str().to_str().unwrap()).italic();

                let inner_area = center_area(
                    content_area,
                    Constraint::Length(1),
                    Constraint::Length(text.width() as u16),
                );

                frame.render_widget(text, inner_area);
            }
            BodyContent::Form(_hash_map) => {
                frame.render_widget(&self.text_editor, content_area);
            }
            BodyContent::Text(_) => {
                frame.render_widget(&self.text_editor, content_area);
            }
        }
    }

    fn draw_overlay(&self, _params: Self::Params, frame: &mut ratatui::Frame) {
        if self.show_dropdown {
            let area = Rect {
                y: self.render_area.top() + 1,
                height: 3,
                ..self.render_area
            };

            frame.render_widget(&self.dropdown, area);
        }
    }
}

impl<'params> Interactive<'params> for BodyEditorComponent {
    type Effect = ();

    type Params = (
        &'params Config,
        &'params mut Events<AppMessage>,
        &'params mut Terminal<CrosstermBackend<Stdout>>,
    );

    fn is_visible_overlay(&self) -> bool {
        self.show_dropdown
    }

    fn handle_key(
        &mut self,
        (config, events, terminal): Self::Params,
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
                        match self.dropdown.selected() {
                            BodyTypeV2::Empty => {
                                self._history.apply(
                                    BodyEditorAction::SetBodyType(BodyContent::Empty),
                                    &mut self.body_content,
                                );
                            }
                            BodyTypeV2::FormUrlEncoded => {
                                self._history.apply(
                                    BodyEditorAction::SetBodyType(
                                        BodyContent::Form(HashMap::new()),
                                    ),
                                    &mut self.body_content,
                                );
                            }
                            BodyTypeV2::Text => {
                                self._history.apply(
                                    BodyEditorAction::SetBodyType(BodyContent::Text(String::new())),
                                    &mut self.body_content,
                                );
                            }
                        }
                        // TODO: delete the text editor lines?
                        self.text_editor.clean_lines();
                    }
                    _ => {}
                }
            }
        } else {
            if self.text_editor.mode().is_write_mode() {
                self.text_editor.handle_key(key);
            } else {
                match config.keymap.match_request_builder_action(key) {
                    Some(action) => match action {
                        keybinding::RequestBuilderKeyAction::OpenDropdown => {
                            self.show_dropdown = true;
                        }
                        keybinding::RequestBuilderKeyAction::OpenEditor => {
                            events.stop();
                            disable_raw_mode().unwrap();
                            stdout().execute(LeaveAlternateScreen).unwrap();

                            self.text_editor.open_in_editor();

                            enable_raw_mode().unwrap();
                            stdout().execute(EnterAlternateScreen).unwrap();
                            let _ = terminal.clear();
                            events.run();
                        }
                    },
                    None => {
                        match &self.body_content {
                            BodyContent::Empty => {}
                            BodyContent::File(_) => {
                                //TODO: Show a menu for select files
                            }
                            _ => {
                                self.text_editor.handle_key(key);
                            }
                        }
                    }
                }
            }
        }
    }
}

enum BodyEditorAction {
    SetBodyType(BodyContent),
}

impl TrackAction for BodyEditorAction {
    type State = BodyContent;

    fn apply(&self, state: &mut Self::State) -> Option<Self>
    where
        Self: Sized,
    {
        match self {
            BodyEditorAction::SetBodyType(body_content) => {
                let previous_body = state.clone();
                *state = body_content.clone();

                Some(BodyEditorAction::SetBodyType(previous_body))
            }
        }
    }
}

impl WithHistory for BodyEditorComponent {
    fn undo(&mut self) {
        self._history.undo(&mut self.body_content);
    }

    fn redo(&mut self) {
        self._history.redo(&mut self.body_content);
    }
}
