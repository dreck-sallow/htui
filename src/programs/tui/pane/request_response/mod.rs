use std::{
    cell::RefCell,
    io::{stdout, Stdout},
    rc::Rc,
    sync::{Arc, RwLock},
};

use arboard::Clipboard;
use body_viewer::BodyContentView;
use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use headers_table::HeadersTable;
use ratatui::{
    layout::{Constraint, Layout, Margin, Rect},
    prelude::CrosstermBackend,
    style::{Style, Stylize},
    symbols::line,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Tabs},
    Terminal,
};
use reqwest::header::CONTENT_TYPE;
use state::RequestResponseState;

use crate::{
    app_project::models::{self, RequestModel, ResponseModel, SendRequest, SendRequestKey},
    programs::tui::{
        common::component::{Drawable, Interactive, Painter},
        config::{keybinding, Config},
        elements::Separator,
        event_handler::{AppMessage, EventSender, Events},
    },
};

use super::{action::PaneAction, ElementFocus};

mod binary_viewer;
mod body_viewer;
mod headers_table;
mod request;
mod state;

#[derive(Clone, Copy, PartialEq)]
pub enum Tab {
    Response,
    Headers,
}

impl Tab {
    pub fn as_idx(&self) -> usize {
        match self {
            Tab::Response => 0,
            Tab::Headers => 1,
        }
    }
}

impl AsRef<str> for Tab {
    fn as_ref(&self) -> &str {
        match self {
            Tab::Headers => "Headers",
            Tab::Response => "Response",
        }
    }
}

pub struct ResponseContent {
    headers_table: HeadersTable,
    body_viewer: BodyContentView,
    request_key: Option<SendRequestKey>,
}

pub struct ResponseViewerComponent {
    state: RequestResponseState,
    tab: Tab,
    response_content: Arc<RwLock<ResponseContent>>,
    clipboard: Rc<RefCell<Clipboard>>,
    config: Rc<Config>,

    status_bar_area: Rect,
    render_area: Rect,
    header_area: Rect,
    content_area: Rect,
}

impl ResponseViewerComponent {
    pub fn new(config: Rc<Config>, clipboard: Rc<RefCell<Clipboard>>) -> Self {
        Self {
            state: RequestResponseState::new(),
            tab: Tab::Response,
            response_content: Arc::new(RwLock::new(ResponseContent {
                headers_table: HeadersTable::new(vec![]),
                body_viewer: BodyContentView::Empty,
                request_key: None,
            })),
            config,
            clipboard,
            status_bar_area: Rect::default(),
            render_area: Rect::default(),
            header_area: Rect::default(),
            content_area: Rect::default(),
        }
    }

    pub fn change_req(&mut self, id: SendRequestKey) {
        let mut locked = self.response_content.write().unwrap();
        locked.request_key = Some(id.clone());

        if let Some(req) = self.state.get(&id) {
            if let SendRequest::Finish(response) = &*req.read().unwrap() {
                request::request_model_to_state(response, &mut *locked);
            }
        }
    }

    pub fn execute_req(&mut self, id: SendRequestKey, req: &RequestModel, sender: EventSender) {
        let send_req = match self.state.get(&id) {
            Some(req) => {
                *req.write().unwrap() = SendRequest::Pending;
                req
            }
            None => Arc::new(RwLock::new(SendRequest::Pending)),
        };

        let send_req_task = request::send_request(
            request::request_into_builder(req),
            request::SendRequestContext {
                request_response: Arc::clone(&send_req),
                current_key: id.clone(),
                response_content: Arc::clone(&self.response_content),
            },
            sender,
        );

        self.state.add_response(id.clone(), send_req, send_req_task);
    }

    fn status_line_ui(&self, res: &ResponseModel) -> Line {
        let status = if res.status >= 400 && res.status < 600 {
            res.status.to_string().red()
        } else if res.status >= 300 {
            res.status.to_string().yellow()
        } else if res.status >= 200 {
            res.status.to_string().green()
        } else {
            res.status.to_string().gray()
        };

        let time_display = {
            let millis = res.duration.as_millis();
            if millis < 1000 {
                format!("Time: {}ms", millis)
            } else {
                format!("Time: {:.2}s", res.duration.as_secs_f32())
            }
        };

        let size_display = {
            let size = res.body.len();
            const KB: usize = 1024;
            const MB: usize = KB * 1024;

            if size < KB {
                format!("Size: {} B", size)
            } else if size < MB {
                format!("Size: {:.2} KB", size as f64 / KB as f64)
            } else {
                format!("Size: {:.2} MB", size as f64 / MB as f64)
            }
        };

        let content_type = res
            .headers
            .iter()
            .find(|(k, _v)| k == CONTENT_TYPE.as_str())
            .map(|(_k, v)| v.to_string())
            .unwrap_or("Unknown".into());

        Line::default().spans([
            Span::from("Status: "),
            status,
            "  ".into(),
            Span::from(time_display),
            "  ".into(),
            Span::from(size_display),
            "  ".into(),
            Span::from(format!("Content-Type: {}", content_type)),
        ])
    }
}

impl Drawable for ResponseViewerComponent {
    type Params = ElementFocus;

    fn set_area(&mut self, area: Rect) {
        let main_areas = Layout::vertical([
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Fill(1),
        ])
        .split(area.inner(Margin::new(1, 1)));

        self.render_area = area;
        self.status_bar_area = main_areas[0];
        self.header_area = main_areas[1];
        self.content_area = main_areas[2];
    }

    fn draw<'a: 'painter, 'painter>(
        &'a self,
        painter: &mut crate::programs::tui::common::component::Painter<'painter>,
        params: Self::Params,
    ) {
        match self
            .response_content
            .read()
            .unwrap()
            .request_key
            .as_ref()
            .and_then(|id| self.state.get(id))
        {
            None => {
                painter.render(|frame| {
                    let placeholder_text = Span::from("Not response yet.");
                    let center_area = {
                        let [area] = Layout::vertical([Constraint::Length(1)])
                            .flex(ratatui::layout::Flex::Center)
                            .areas(self.render_area);

                        let [area] = Layout::horizontal([Constraint::Length(
                            placeholder_text.width() as u16,
                        )])
                        .flex(ratatui::layout::Flex::Center)
                        .areas(area);

                        area
                    };
                    frame.render_widget(placeholder_text, center_area);
                });
            }
            Some(send_request_response) => match &*send_request_response.read().unwrap() {
                models::SendRequest::Pending => {
                    painter.render(|frame| {
                        let placeholder_text = Span::from("Sending...");
                        let center_area = {
                            let [area] = Layout::vertical([Constraint::Length(1)])
                                .flex(ratatui::layout::Flex::Center)
                                .areas(self.render_area);

                            let [area] = Layout::horizontal([Constraint::Length(
                                placeholder_text.width() as u16,
                            )])
                            .flex(ratatui::layout::Flex::Center)
                            .areas(area);

                            area
                        };

                        frame.render_widget(placeholder_text, center_area);
                    });
                }
                models::SendRequest::Finish(response) => {
                    // Draw the status line
                    let status_line = self.status_line_ui(response);

                    painter.render(move |frame| {
                        let is_focus = params == ElementFocus::ResponseViewer;
                        let border_style = Style::default().fg(if is_focus {
                            self.config.theme.border_focus
                        } else {
                            self.config.theme.border
                        });

                        let border_type = if is_focus {
                            BorderType::Thick
                        } else {
                            BorderType::Plain
                        };

                        let block = Block::bordered()
                            .border_type(border_type)
                            .border_style(border_style);
                        frame.render_widget(block, self.render_area);

                        // Draw the status line bar
                        frame.render_widget(
                            &status_line,
                            Rect {
                                height: 1,
                                ..self.status_bar_area
                            },
                        );
                        frame.render_widget(
                            Separator::default()
                                .symbol(line::HORIZONTAL)
                                .style(border_style),
                            Rect {
                                height: 1,
                                y: self.status_bar_area.top() + 1,
                                ..self.status_bar_area
                            },
                        );

                        let tab_titles = {
                            let locked = self.response_content.read().unwrap();
                            let headers_count = locked.headers_table.len_items();
                            [
                                format!(" {} ", Tab::Response.as_ref()),
                                format!(" {} ({})", Tab::Headers.as_ref(), headers_count),
                            ]
                        };

                        let tabs = Tabs::new(tab_titles)
                            .select(self.tab.as_idx())
                            .block(
                                Block::new()
                                    .borders(Borders::BOTTOM)
                                    .border_type(border_type)
                                    .border_style(border_style),
                            )
                            .highlight_style(Style::default().fg(self.config.theme.tab_highlight));

                        frame.render_widget(tabs, self.header_area);
                    });

                    match self.tab {
                        Tab::Headers => {
                            let response_content = Arc::clone(&self.response_content);
                            painter.render(move |frame| {
                                let locked = response_content.read().unwrap();
                                let table_ui = locked.headers_table.table_ui(&self.config.theme);
                                frame.render_widget(table_ui, self.content_area);
                            });
                        }
                        Tab::Response => {
                            let response_content = Arc::clone(&self.response_content);

                            painter.render(move |frame| {
                                // FIXME: use the same painter from the draw tree call (because on overlays cannot work in nested painters)
                                let locked = response_content.read().unwrap();
                                let mut _painter = Painter::new();
                                locked.body_viewer.draw(
                                    &mut _painter,
                                    (self.content_area, Rc::clone(&self.config)),
                                );
                                _painter.draw(frame);
                            });
                        }
                    }
                }
            },
        }
    }
}

impl Interactive for ResponseViewerComponent {
    type Effect = PaneAction;
    type Params = (
        Rc<RefCell<Events<AppMessage>>>,
        Rc<RefCell<Terminal<CrosstermBackend<Stdout>>>>,
    );

    fn on_key(
        &mut self,
        key: crossterm::event::KeyEvent,
        (events, terminal): Self::Params,
    ) -> Option<Self::Effect> {
        let is_consumed = match self.config.keymap.match_global_action(key) {
            Some(action) => match action {
                keybinding::GlobalKeyAction::NextFocus => return Some(PaneAction::NextFocus),
                keybinding::GlobalKeyAction::PreviousFocus => {
                    return Some(PaneAction::PreviousFocus)
                }
                keybinding::GlobalKeyAction::NextTab => {
                    if Tab::Response == self.tab {
                        self.tab = Tab::Headers
                    }
                    true
                }
                keybinding::GlobalKeyAction::PreviousTab => {
                    if Tab::Headers == self.tab {
                        self.tab = Tab::Response
                    }
                    true
                }
                _ => false,
            },
            None => false,
        };

        if !is_consumed {
            match self.tab {
                Tab::Headers => {
                    let mut response_content = self.response_content.write().unwrap();
                    let headers_editor = &mut response_content.headers_table;
                    if let Some(key) = self.config.keymap.match_global_action(key) {
                        match key {
                            keybinding::GlobalKeyAction::MoveDown => {
                                headers_editor.move_row_idx(true)
                            }
                            keybinding::GlobalKeyAction::MoveUp => {
                                headers_editor.move_row_idx(false)
                            }
                            keybinding::GlobalKeyAction::MoveLeft => {
                                headers_editor.move_col_idx(false)
                            }
                            keybinding::GlobalKeyAction::MoveRight => {
                                headers_editor.move_col_idx(true)
                            }
                            keybinding::GlobalKeyAction::CopyToClipboard => {
                                if let Some(txt) = headers_editor.cell_txt() {
                                    self.clipboard.borrow_mut().set_text(txt).unwrap();
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Tab::Response => {
                    let mut response_content = self.response_content.write().unwrap();
                    let body_viewer = &mut response_content.body_viewer;
                    match body_viewer {
                        BodyContentView::Text(text_editor) => {
                            match self.config.keymap.match_request_builder_action(key) {
                                Some(action) => match action {
                                    keybinding::RequestBuilderKeyAction::OpenEditor => {
                                        events.borrow_mut().stop();
                                        disable_raw_mode().unwrap();
                                        stdout().execute(LeaveAlternateScreen).unwrap();

                                        text_editor.open_in_editor();

                                        enable_raw_mode().unwrap();
                                        stdout().execute(EnterAlternateScreen).unwrap();
                                        let _ = terminal.borrow_mut().clear();
                                        events.borrow_mut().run();
                                    }
                                    _ => text_editor.handle_key(key),
                                },
                                None => text_editor.handle_key(key),
                            }
                        }
                        BodyContentView::Binary(binary_viewer) => {
                            binary_viewer
                                .on_key(key, (Rc::clone(&self.config), Rc::clone(&self.clipboard)));
                        }
                        BodyContentView::Empty => {}
                    }
                }
            }
        }

        None
    }
}
