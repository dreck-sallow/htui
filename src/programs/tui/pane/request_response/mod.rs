use std::{
    io::{stdout, Stdout},
    path::PathBuf,
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
    app_project::models::{
        self, RequestModel, ResponseFilePath, ResponseModel, SendRequest, SendRequestKey,
    },
    programs::tui::{
        common::{Interactive, UiComposedElement},
        config::{keybinding, Config},
        elements::Separator,
        event_handler::{AppMessage, EventSender, Events},
    },
};

use super::{action::PaneAction, ElementFocus};

mod binary_body;
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
    // config: Rc<Config>,
    status_bar_area: Rect,
    render_area: Rect,
    header_area: Rect,
    content_area: Rect,
}

impl ResponseViewerComponent {
    pub fn new() -> Self {
        Self {
            state: RequestResponseState::new(),
            tab: Tab::Response,
            response_content: Arc::new(RwLock::new(ResponseContent {
                headers_table: HeadersTable::new(vec![]),
                body_viewer: BodyContentView::empty(Rect::default()),
                request_key: None,
            })),
            // config,
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
                request::request_model_to_state(response, &mut *locked, self.content_area);
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
            self.content_area,
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

impl<'params> UiComposedElement<'params> for ResponseViewerComponent {
    type Params = (ElementFocus, &'params Config);

    fn set_area(&mut self, area: Rect, _viewport_area: Rect) {
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
        self.response_content
            .write()
            .unwrap()
            .body_viewer
            .set_area(main_areas[2], _viewport_area);
    }

    fn draw(&self, (focus, config): Self::Params, frame: &mut ratatui::Frame) {
        match self
            .response_content
            .read()
            .unwrap()
            .request_key
            .as_ref()
            .and_then(|id| self.state.get(id))
        {
            None => {
                let placeholder_text = Span::from("Not response yet.");
                let center_area = {
                    let [area] = Layout::vertical([Constraint::Length(1)])
                        .flex(ratatui::layout::Flex::Center)
                        .areas(self.render_area);

                    let [area] =
                        Layout::horizontal([Constraint::Length(placeholder_text.width() as u16)])
                            .flex(ratatui::layout::Flex::Center)
                            .areas(area);

                    area
                };
                frame.render_widget(placeholder_text, center_area);
            }
            Some(send_request_response) => match &*send_request_response.read().unwrap() {
                models::SendRequest::Pending => {
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
                }
                models::SendRequest::Finish(response) => {
                    // Draw the status line
                    let status_line = self.status_line_ui(response);

                    let is_focus = focus == ElementFocus::ResponseViewer;
                    let border_style = Style::default().fg(if is_focus {
                        config.theme.border_focus
                    } else {
                        config.theme.border
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
                        .highlight_style(Style::default().fg(config.theme.tab_highlight));

                    frame.render_widget(tabs, self.header_area);

                    match self.tab {
                        Tab::Headers => {
                            let response_content = Arc::clone(&self.response_content);
                            let locked = response_content.read().unwrap();
                            let table_ui = locked.headers_table.table_ui(&config.theme);
                            frame.render_widget(table_ui, self.content_area);
                        }
                        Tab::Response => {
                            // FIXME: use the same painter from the draw tree call (because on overlays cannot work in nested painters)
                            let locked = self.response_content.read().unwrap();
                            locked.body_viewer.draw(config, frame);
                        }
                    }
                }
            },
        }
    }

    fn draw_overlay(&self, (_, config): Self::Params, frame: &mut ratatui::Frame) {
        if let Some(send_request_response) = self
            .response_content
            .read()
            .unwrap()
            .request_key
            .as_ref()
            .and_then(|id| self.state.get(id))
        {
            match &*send_request_response.read().unwrap() {
                models::SendRequest::Pending => {}
                models::SendRequest::Finish(_) => match self.tab {
                    Tab::Headers => {
                        let locked = self.response_content.read().unwrap();
                        let table_ui = locked.headers_table.table_ui(&config.theme);
                        frame.render_widget(table_ui, self.content_area);
                    }
                    Tab::Response => {
                        let locked = self.response_content.read().unwrap();
                        locked.body_viewer.draw_overlay(config, frame);
                    }
                },
            }
        }
    }
}

impl<'params> Interactive<'params> for ResponseViewerComponent {
    type Effect = PaneAction;

    type Params = (
        &'params Config,
        &'params mut Events<AppMessage>,
        &'params mut Terminal<CrosstermBackend<Stdout>>,
        &'params mut Clipboard,
    );

    fn handle_key(
        &mut self,
        (config, events, terminal, clipboard): Self::Params,
        key: crossterm::event::KeyEvent,
    ) -> Self::Effect {
        let is_consumed = match config.keymap.match_global_action(key) {
            Some(action) => match action {
                keybinding::GlobalKeyAction::NextFocus => return PaneAction::NextFocus,
                keybinding::GlobalKeyAction::PreviousFocus => return PaneAction::PreviousFocus,
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
                    if let Some(key) = config.keymap.match_global_action(key) {
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
                                    clipboard.set_text(txt).unwrap();
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Tab::Response => {
                    let mut response_content = self.response_content.write().unwrap();
                    let request_key = response_content.request_key.clone().unwrap();
                    let body_viewer = &mut response_content.body_viewer;
                    match body_viewer {
                        BodyContentView::Text { editor, .. } => {
                            match config.keymap.match_request_builder_action(key) {
                                Some(action) => match action {
                                    keybinding::RequestBuilderKeyAction::OpenEditor => {
                                        events.stop();
                                        disable_raw_mode().unwrap();
                                        stdout().execute(LeaveAlternateScreen).unwrap();

                                        editor.open_in_editor();

                                        enable_raw_mode().unwrap();
                                        stdout().execute(EnterAlternateScreen).unwrap();
                                        let _ = terminal.clear();
                                        events.run();
                                    }
                                    _ => editor.handle_key(key),
                                },
                                None => editor.handle_key(key),
                            }
                        }
                        BodyContentView::Binary(binary_viewer) => {
                            let send_request_state = self.state.get(&request_key).unwrap();
                            let mut request_model = send_request_state.write().unwrap();

                            let slice_bytes = match &*request_model {
                                SendRequest::Pending => unreachable!(),
                                SendRequest::Finish(response_model) => &response_model.body,
                            };

                            match binary_viewer.handle_key((config, clipboard, slice_bytes), key) {
                                binary_body::BinaryViewerEffect::NewFilePath(path) => {
                                    match &mut *request_model {
                                        SendRequest::Pending => {}
                                        SendRequest::Finish(response_model) => {
                                            response_model.file_path =
                                                ResponseFilePath::Saved(PathBuf::from(path));
                                        }
                                    }
                                }
                                binary_body::BinaryViewerEffect::Noop => {}
                            }
                        }
                        BodyContentView::Empty(_) => {}
                    }
                }
            }
        }
        PaneAction::Noop
    }
}
