use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout, Margin, Rect},
    style::{Style, Stylize},
    text::Span,
    widgets::{Block, Borders, Padding, Tabs},
    Frame,
};
use request_builder_state::{Focus, ViewTab};
use tui_textarea::{Input, TextArea};

use crate::programs::tui::element_view::ElementView;

use super::{editor::TextEditor, global_pane_state::GlobalPaneState};

pub mod request_builder_state;

#[derive(Default)]
struct ChildrenAreas {
    method_area: Rect,
    input_area: Rect,
    tabs_area: Rect,
    pane_area: Rect,
}

impl ChildrenAreas {
    pub fn calculate(&mut self, area: Rect) {
        let [header_area, content_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);

        let [method_area, input_area] =
            Layout::horizontal([Constraint::Length(6), Constraint::Fill(1)]).areas(header_area);

        let [tabs_area, pane_area] =
            Layout::vertical([Constraint::Length(2), Constraint::Fill(1)]).areas(content_area);

        self.method_area = method_area;
        self.input_area = input_area;
        self.tabs_area = tabs_area;
        self.pane_area = pane_area;
    }
}

pub struct RequestBuilderView {
    render_area: Rect,
    children_areas: ChildrenAreas,
    url_input: TextArea<'static>,
    header_editor: TextEditor,
    body_editor: TextArea<'static>,
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
            render_area: Rect::default(),
            children_areas: ChildrenAreas::default(),
            url_input,
            header_editor: TextEditor::new(true),
            body_editor: editor,
        }
    }

    fn handle_key_for_method(&mut self, key: KeyEvent, state: &mut GlobalPaneState) {
        match key.code {
            KeyCode::Enter => {
                // Execute request
                // if let Some(_coord) = self.__method_area {
                //     state.set_overlay(super::focus::OverlayFocus::MethodSelector);
                //     // register_actions.push(Action::SelectMethod(coord, HttpMethod::Head));
                // }
            }
            KeyCode::Tab => {
                state.builder_state_mut().next_focus();
            }
            KeyCode::BackTab => {
                state.set_focus(super::focus::ElementFocus::Collections);
            }
            _ => {}
        }
    }

    fn handle_key_for_url_iput(&mut self, key: KeyEvent, state: &mut GlobalPaneState) {
        match key.code {
            KeyCode::Enter => {
                // Execute request
            }
            KeyCode::Tab => {
                state.builder_state_mut().next_focus();
            }
            KeyCode::BackTab => {
                state.builder_state_mut().prev_focus();
            }
            _ => {
                let input = Input::from(key);
                self.url_input.input(input);
            }
        }
    }

    fn handle_key_for_tabs(&mut self, key: KeyEvent, state: &mut GlobalPaneState) {
        match key.code {
            KeyCode::Tab => {
                state.builder_state_mut().next_focus();
            }
            KeyCode::BackTab => {
                state.builder_state_mut().prev_focus();
            }
            KeyCode::Left | KeyCode::Char('h') => state.builder_state_mut().prev_tab(),
            KeyCode::Right | KeyCode::Char('l') => state.builder_state_mut().next_tab(),
            _ => {}
        }
    }

    fn handle_key_for_content(&mut self, key: KeyEvent, state: &mut GlobalPaneState) {
        match key.code {
            KeyCode::Tab => {
                state.set_focus(super::focus::ElementFocus::ResponseViewer);
            }
            KeyCode::BackTab => {
                state.builder_state_mut().prev_focus();
            }
            _ => {
                let input = Input::from(key);

                match state.builder_state_ref().view_tab() {
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
}

impl ElementView for RequestBuilderView {
    type State = GlobalPaneState;

    fn draw(&self, frame: &mut Frame, state: &Self::State) {
        // SAFETY: check on the parent for render this section

        let request = state.current_request().unwrap();
        let is_focus = state.is_focus(super::focus::ElementFocus::RequestBuilder);
        let block = Block::bordered()
            .title(" Request builder ")
            .border_style(
                is_focus
                    .then_some(Style::default().blue())
                    .unwrap_or_default(),
            )
            .padding(Padding::symmetric(1, 0));

        frame.render_widget(block, self.render_area);

        let builder_state = state.builder_state_ref();

        frame.render_widget(
            Span::from(request.method().as_ref()).style(Style::new().blue()),
            self.children_areas.method_area,
        );

        frame.render_widget(&self.url_input, self.children_areas.input_area);

        let tabs = Tabs::new([
            format!(" {} ", ViewTab::Headers.to_string()),
            format!(" {} ", ViewTab::Body.to_string()),
        ])
        .select(builder_state.view_tab_index() as usize)
        .highlight_style(
            (builder_state.focus() == &Focus::Tabs)
                .then_some(Style::default().on_light_magenta().black())
                .unwrap_or(Style::default().on_dark_gray()),
        )
        .block(Block::new().borders(Borders::BOTTOM));

        frame.render_widget(tabs, self.children_areas.tabs_area);
        match builder_state.view_tab() {
            ViewTab::Headers => {
                frame.render_widget(&self.header_editor, self.children_areas.pane_area)
            }
            ViewTab::Body => frame.render_widget(&self.body_editor, self.children_areas.pane_area),
        }
    }

    fn set_area(&mut self, area: Rect) {
        self.children_areas.calculate(area.inner(Margin::new(1, 1)));
        self.render_area = area;
    }

    fn on_key(&mut self, key: KeyEvent, state: &mut Self::State) {
        if key.kind == KeyEventKind::Press {
            match state.builder_state_ref().focus() {
                request_builder_state::Focus::Method => {
                    self.handle_key_for_method(key, state);
                }
                request_builder_state::Focus::UrlInput => {
                    self.handle_key_for_url_iput(key, state);
                }
                request_builder_state::Focus::Tabs => {
                    self.handle_key_for_tabs(key, state);
                }
                request_builder_state::Focus::TabContent => {
                    self.handle_key_for_content(key, state);
                }
            }
        }
    }

    fn on_change_state(&mut self, _state: &Self::State) {}
}
