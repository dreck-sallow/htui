use std::{
    cell::RefCell,
    collections::HashMap,
    io::{stdout, Stdout},
    rc::Rc,
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
            component::{Drawable, Interactive, WithHistory},
        },
        config::{keybinding, Config},
        elements::dropdown::OverlayDropdown,
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
    config: Rc<Config>,
    _history: ActionHistory<BodyEditorAction>,
}

impl BodyEditorComponent {
    pub fn new(config: Rc<Config>) -> Self {
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
            config,
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
    }

    pub fn get_data(&self) -> BodyContent {
        self.body_content.clone()
    }
}

impl Drawable for BodyEditorComponent {
    type Params = ();

    fn set_area(&mut self, _area: Rect) {
        self.render_area = _area;
    }

    fn draw<'a: 'painter, 'painter>(
        &'a self,
        painter: &mut crate::programs::tui::common::component::Painter<'painter>,
        _params: Self::Params,
    ) {
        painter.render(|frame| {
            let [header_area, content_area] =
                Layout::vertical([Constraint::Length(1), Constraint::Fill(1)])
                    .areas(self.render_area);

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
                Style::default().fg(self.config.theme.dropdown.fg).bg(self
                    .config
                    .theme
                    .dropdown
                    .bg),
            );

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
                    height: 3,
                    ..self.render_area
                };

                frame.render_widget(&self.dropdown, area);
            });
        }
    }
}

impl Interactive for BodyEditorComponent {
    type Effect = ();
    type Params = (
        Rc<RefCell<Events<AppMessage>>>,
        Rc<RefCell<Terminal<CrosstermBackend<Stdout>>>>,
    );

    fn on_key(
        &mut self,
        key: crossterm::event::KeyEvent,
        (events, terminal): Self::Params,
    ) -> Option<Self::Effect> {
        if self.show_dropdown {
            if let Some(key) = self.config.keymap.match_global_action(key) {
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
                match self.config.keymap.match_request_builder_action(key) {
                    Some(action) => match action {
                        keybinding::RequestBuilderKeyAction::OpenDropdown => {
                            self.show_dropdown = true;
                            // self.dropdown.select(BodyTypeV2::Empty);
                        }
                        keybinding::RequestBuilderKeyAction::OpenEditor => {
                            events.borrow_mut().stop();
                            disable_raw_mode().unwrap();
                            stdout().execute(LeaveAlternateScreen).unwrap();

                            self.text_editor.open_in_editor();

                            enable_raw_mode().unwrap();
                            stdout().execute(EnterAlternateScreen).unwrap();
                            let _ = terminal.borrow_mut().clear();
                            events.borrow_mut().run();
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
        None
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

// pub fn edit_text_on_editor(text: &str) -> String {
//     disable_raw_mode().unwrap();
//     stdout().execute(LeaveAlternateScreen).unwrap();

//     // Start the editing
//     let mut file = NamedTempFile::new().unwrap();
//     file.write_all(text.as_bytes()).unwrap();

//     let file_path = file.into_temp_path();

//     Command::new(var("EDITOR").unwrap())
//         .args([&file_path])
//         .status()
//         .expect("Error executing editor");
//     // file_path.close();
//     enable_raw_mode().unwrap();
//     stdout().execute(EnterAlternateScreen).unwrap();
//     fs::read_to_string(file_path).unwrap()
// }
