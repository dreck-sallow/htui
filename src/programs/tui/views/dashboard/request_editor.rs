use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout, Margin, Rect},
    style::{Style, Stylize},
    widgets::{Block, Borders, Tabs},
};

use crate::{programs::tui::element_view::ElementView, store::models::BodyContent};

use super::{
    editor::TextEditor, global_pane_state::GlobalPaneState,
    http_payload_editor::body_editor::BodyEditor,
};

#[derive(Clone, Copy)]
pub enum RequestTab {
    Headers,
    Body,
}

impl Into<&str> for RequestTab {
    fn into(self) -> &'static str {
        match self {
            RequestTab::Headers => " Headers ",
            RequestTab::Body => " Body ",
        }
    }
}

pub struct RequestEditorView {
    render_area: Rect,
    header_area: Rect,
    content_area: Rect,
    tab: RequestTab,
    headers_editor: TextEditor,
    body_editor: BodyEditor,
    request_idx: (usize, usize),
}

impl RequestEditorView {
    pub fn new() -> Self {
        Self {
            render_area: Rect::default(),
            header_area: Rect::default(),
            content_area: Rect::default(),
            tab: RequestTab::Headers,
            headers_editor: TextEditor::new(true),
            body_editor: BodyEditor::new(true),
            request_idx: (0, 0),
        }
    }

    pub fn tab_idx(&self) -> usize {
        match self.tab {
            RequestTab::Headers => 0,
            RequestTab::Body => 1,
        }
    }
}

impl ElementView for RequestEditorView {
    type State = GlobalPaneState;

    fn draw(&self, frame: &mut ratatui::Frame, state: &Self::State) {
        let style = if state.is_focus(super::focus::ElementFocus::RequestBuilder) {
            Style::default().blue()
        } else {
            Style::default()
        };
        let block = Block::bordered().border_style(style);
        frame.render_widget(block, self.render_area);

        let tabs = Tabs::new([
            Into::<&str>::into(RequestTab::Headers),
            Into::<&str>::into(RequestTab::Body),
        ])
        .select(self.tab_idx())
        .block(Block::new().borders(Borders::BOTTOM).border_style(style))
        .highlight_style(Style::default().blue());

        frame.render_widget(tabs, self.header_area);

        match self.tab {
            RequestTab::Headers => {
                frame.render_widget(&self.headers_editor, self.content_area);
            }
            RequestTab::Body => {
                self.body_editor.draw(frame, &BodyContent::Empty);
            }
        }
    }

    fn on_key(&mut self, key: crossterm::event::KeyEvent, state: &mut Self::State) {
        if key.kind == KeyEventKind::Press {
            let is_editing = match self.tab {
                RequestTab::Headers => self.headers_editor.mode().is_write_mode(),
                RequestTab::Body => false,
            };

            match key.code {
                KeyCode::Tab => match self.tab {
                    RequestTab::Headers => {
                        if is_editing {
                            self.headers_editor.handle_key(key)
                        } else {
                            self.tab = RequestTab::Body;
                        }
                    }
                    RequestTab::Body => {
                        if is_editing {
                            self.body_editor.on_key(key, &mut BodyContent::Empty);
                        } else {
                            state.set_focus(super::focus::ElementFocus::ResponseViewer);
                        }
                    }
                },

                KeyCode::BackTab => match self.tab {
                    RequestTab::Headers => {
                        if is_editing {
                            self.headers_editor.handle_key(key);
                        } else {
                            state.set_focus(super::focus::ElementFocus::MethodUrlBar);
                        }
                    }
                    RequestTab::Body => {
                        if is_editing {
                            self.body_editor.on_key(key, &mut BodyContent::Empty);
                        } else {
                            self.tab = RequestTab::Headers;
                        }
                    }
                },
                _ => match self.tab {
                    RequestTab::Headers => self.headers_editor.handle_key(key),
                    RequestTab::Body => self.body_editor.on_key(
                        key,
                        &mut BodyContent::Text(String::from(
                            "{'name': 'dikson Aranda'\n, 'age': 'other name'}",
                        )),
                    ),
                },
            }
        }
    }

    fn set_area(&mut self, area: Rect) {
        let main_areas = Layout::vertical([Constraint::Length(2), Constraint::Fill(50)])
            .split(area.inner(Margin::new(1, 1)));

        self.render_area = area;
        self.header_area = main_areas[0];
        self.content_area = main_areas[1];
        self.body_editor.set_area(main_areas[1]);
    }

    fn on_change_state(&mut self, state: &Self::State) {
        match state.current_request_idx {
            Some(idx) => {
                if self.request_idx != idx {
                    // Save the request data into "Request"

                    // Start from zero
                    {
                        let mut headers_text = String::new();
                        for (k, v) in state.current_request().unwrap().headers() {
                            headers_text.push_str(&format!("{k}:{v}\n"));
                        }

                        self.headers_editor.insert_str(&headers_text);
                    }

                    self.tab = RequestTab::Headers;
                    self.request_idx = idx;
                }
            }
            None => {}
        }
    }
}
