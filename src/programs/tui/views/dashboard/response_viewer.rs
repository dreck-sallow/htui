use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout, Margin, Rect},
    style::{Style, Stylize},
    widgets::{Block, Borders, Tabs},
};

use crate::{programs::tui::element_view::ElementView, store::models::BodyContent};

use super::{
    editor::TextEditor,
    global_pane_state::GlobalPaneState,
    http_payload_editor::body_editor::BodyEditor,
    pane_state::{history::MutationCollector, mutations::SetFocus},
};

#[derive(Clone, Copy)]
pub enum ResponseTab {
    Headers,
    Response,
}

impl Into<&str> for ResponseTab {
    fn into(self) -> &'static str {
        match self {
            ResponseTab::Headers => " Headers ",
            ResponseTab::Response => " Response ",
        }
    }
}

pub struct ResponseViewerView {
    render_area: Rect,
    header_area: Rect,
    content_area: Rect,
    tab: ResponseTab,
    headers_editor: TextEditor,
    body_editor: BodyEditor,
    request_idx: (usize, usize),
}

impl ResponseViewerView {
    pub fn new() -> Self {
        Self {
            render_area: Rect::default(),
            header_area: Rect::default(),
            content_area: Rect::default(),
            tab: ResponseTab::Response,
            headers_editor: TextEditor::new(false),
            body_editor: BodyEditor::new_readable(),
            request_idx: (0, 0),
        }
    }

    pub fn tab_idx(&self) -> usize {
        match self.tab {
            ResponseTab::Headers => 1,
            ResponseTab::Response => 0,
        }
    }
}

impl<'a> ElementView<'a> for ResponseViewerView {
    type State = GlobalPaneState;
    type Collector = MutationCollector<'a>;

    fn draw(&self, frame: &mut ratatui::Frame, state: &Self::State) {
        let style = if state.is_focus(super::focus::ElementFocus::ResponseViewer) {
            Style::default().blue()
        } else {
            Style::default()
        };
        let block = Block::bordered().border_style(style);
        frame.render_widget(block, self.render_area);

        let tabs = Tabs::new([
            Into::<&str>::into(ResponseTab::Response),
            Into::<&str>::into(ResponseTab::Headers),
        ])
        .select(self.tab_idx())
        .block(Block::new().borders(Borders::BOTTOM).border_style(style))
        .highlight_style(Style::default().blue());

        frame.render_widget(tabs, self.header_area);

        match self.tab {
            ResponseTab::Headers => {
                frame.render_widget(&self.headers_editor, self.content_area);
            }
            ResponseTab::Response => {
                self.body_editor.draw(frame, &BodyContent::Empty);
            }
        }
    }

    fn on_key(&mut self, key: crossterm::event::KeyEvent, collector: &mut Self::Collector) {
        if key.kind == KeyEventKind::Press {
            let is_editing = match self.tab {
                ResponseTab::Headers => self.headers_editor.mode().is_write_mode(),
                ResponseTab::Response => false,
            };

            match key.code {
                KeyCode::Tab => match self.tab {
                    ResponseTab::Headers => {
                        if is_editing {
                            self.headers_editor.handle_key(key)
                        } else {
                            collector.add(SetFocus::for_next());
                        }
                    }
                    ResponseTab::Response => {
                        if is_editing {
                            // self.body_editor.on_key(key, &mut BodyContent::Empty);
                        } else {
                            self.tab = ResponseTab::Headers;
                        }
                    }
                },

                KeyCode::BackTab => match self.tab {
                    ResponseTab::Headers => {
                        if is_editing {
                            self.headers_editor.handle_key(key);
                        } else {
                            self.tab = ResponseTab::Response;
                        }
                    }
                    ResponseTab::Response => {
                        if is_editing {
                            // self.body_editor.on_key(key, &mut BodyContent::Empty);
                        } else {
                            collector.add(SetFocus::for_previous());
                        }
                    }
                },
                _ => match self.tab {
                    ResponseTab::Headers => self.headers_editor.handle_key(key),
                    ResponseTab::Response => {
                        // self.body_editor.on_key(
                        //                         key,
                        //                         &mut BodyContent::Text(String::from(
                        //                             "{'name': 'dikson Aranda'\n, 'age': 'other name'}",
                        //                         )),
                        //                     )
                    }
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
                    self.headers_editor.clean_lines();
                    {
                        let mut headers_text = String::new();
                        for (k, v) in state.current_request().unwrap().headers() {
                            headers_text.push_str(&format!("{k}:{v}\n"));
                        }

                        self.headers_editor.insert_str(&headers_text);
                    }

                    self.tab = ResponseTab::Headers;
                    self.request_idx = idx;
                }
            }
            None => {}
        }
    }
}
