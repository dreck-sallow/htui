use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::Span,
    widgets::{Block, Borders, Padding, Tabs},
    Frame,
};
use request_builder_state::{Focus, RequestBuilderState, ViewTab};
use tui_textarea::{Input, TextArea};

use crate::store::models::HttpMethod;

use super::{action::Action, editor::TextEditor, pane_state::PaneState};

pub mod request_builder_state;

pub struct RequestBuilderView {
    state: RequestBuilderState,
    url_input: TextArea<'static>,
    header_editor: TextEditor,
    body_editor: TextArea<'static>,
    __method_area: Option<(u16, u16)>,
}

impl RequestBuilderView {
    pub fn new() -> Self {
        let mut url_input = TextArea::default();

        url_input.set_cursor_line_style(Style::default());
        url_input.set_placeholder_text("Enter a url");
        url_input.insert_str("https://");

        let mut editor = TextArea::default();
        editor.set_cursor_line_style(Style::default());

        Self {
            state: RequestBuilderState::new(),
            url_input,
            header_editor: TextEditor::new(),
            body_editor: editor,
            __method_area: None,
        }
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect, is_focus: bool) {
        let block = Block::bordered()
            .title(" Request builder ")
            .border_style(
                is_focus
                    .then_some(Style::default().blue())
                    .unwrap_or_default(),
            )
            .padding(Padding::symmetric(1, 0));

        let [method_area, input_area, tabs_area, pane_area] = {
            let [header_area, content_area] =
                Layout::vertical([Constraint::Length(1), Constraint::Fill(1)])
                    .areas(block.inner(area));

            let [method_area, input_area] =
                Layout::horizontal([Constraint::Length(6), Constraint::Fill(1)]).areas(header_area);

            let [tabs_area, pane_area] =
                Layout::vertical([Constraint::Length(2), Constraint::Fill(1)]).areas(content_area);

            [method_area, input_area, tabs_area, pane_area]
        };

        frame.render_widget(block, area);

        frame.render_widget(
            Span::from(Into::<&str>::into(&self.state.method)).style(Style::new().blue()),
            method_area,
        );

        // TODO: handle better this way of store the areas!
        self.__method_area = Some((method_area.left(), method_area.bottom()));

        self.url_input.set_style(
            (self.state.focus() == &Focus::UrlInput)
                .then_some(Style::default().magenta())
                .unwrap_or_default()
                .on_dark_gray(),
        );
        frame.render_widget(&self.url_input, input_area);

        let tabs = Tabs::new([
            format!(" {} ", ViewTab::Headers.to_string()),
            format!(" {} ", ViewTab::Body.to_string()),
        ])
        .select(self.state.view_tab_index() as usize)
        .highlight_style(
            (self.state.focus() == &Focus::Tabs)
                .then_some(Style::default().on_light_magenta().black())
                .unwrap_or(Style::default().on_dark_gray()),
        )
        .block(Block::new().borders(Borders::BOTTOM));

        frame.render_widget(tabs, tabs_area);
        match self.state.view_tab() {
            ViewTab::Headers => frame.render_widget(&self.header_editor, pane_area),
            ViewTab::Body => frame.render_widget(&self.body_editor, pane_area),
        }
    }

    fn handle_key_for_method(
        &mut self,
        key: KeyEvent,
        state: &mut PaneState,
        register_actions: &mut Vec<Action>,
    ) {
        match key.code {
            KeyCode::Enter => {
                // Execute request
                if let Some(coord) = self.__method_area {
                    state.focus_overlay(super::focus::OverlayFocus::MethodSelector);
                    register_actions.push(Action::SelectMethod(coord, HttpMethod::Head));
                }
            }
            KeyCode::Tab => {
                self.state.next_focus();
            }
            KeyCode::BackTab => {
                state.prev_focus_element();
            }
            _ => {}
        }
    }

    fn handle_key_for_url_iput(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter => {
                // Execute request
            }
            KeyCode::Tab => {
                self.state.next_focus();
            }
            KeyCode::BackTab => {
                self.state.prev_focus();
            }
            _ => {
                let input = Input::from(key);
                self.url_input.input(input);
            }
        }
    }

    fn handle_key_for_tabs(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Tab => {
                self.state.next_focus();
            }
            KeyCode::BackTab => {
                self.state.prev_focus();
            }
            KeyCode::Left | KeyCode::Char('h') => self.state.prev_tab(),
            KeyCode::Right | KeyCode::Char('l') => self.state.next_tab(),
            _ => {}
        }
    }

    fn handle_key_for_content(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Tab => {
                self.state.next_focus();
            }
            KeyCode::BackTab => {
                self.state.prev_focus();
            }
            _ => {
                let input = Input::from(key);

                match self.state.view_tab() {
                    ViewTab::Headers => {
                        self.header_editor.handle_key(key);
                    }
                    ViewTab::Body => {
                        self.body_editor.input(input);
                    }
                }
            }
        }
    }

    pub fn handle_key(
        &mut self,
        key: KeyEvent,
        state: &mut PaneState,
        register_actions: &mut Vec<Action>,
    ) {
        if key.kind == KeyEventKind::Press {
            match self.state.focus() {
                request_builder_state::Focus::Method => {
                    self.handle_key_for_method(key, state, register_actions);
                }
                request_builder_state::Focus::UrlInput => {
                    self.handle_key_for_url_iput(key);
                }
                request_builder_state::Focus::Tabs => {
                    self.handle_key_for_tabs(key);
                }
                request_builder_state::Focus::TabContent => {
                    self.handle_key_for_content(key);
                }
            }
        }
    }

    pub fn handle_action(&mut self, action: Action) {
        match action {
            Action::SelectedMethod(http_method) => {
                self.state.method = http_method;
            }
            _ => {}
        }
    }
}
