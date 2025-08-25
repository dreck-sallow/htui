use std::{
    cell::RefCell,
    io::{stdout, Stdout},
    rc::Rc,
    sync::{Arc, RwLock},
};

use arboard::Clipboard;
use binary_viewer::BinaryViewer;
use body_viewer::BodyContentView;
use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use encoding_rs::{Encoding, UTF_8};
use headers_table::HeadersTable;
use mime::Mime;
use ratatui::{
    layout::{Constraint, Layout, Margin, Rect},
    prelude::CrosstermBackend,
    style::{Style, Stylize},
    symbols::line,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Tabs},
    Terminal,
};
use reqwest::{header::CONTENT_TYPE, ClientBuilder, RequestBuilder, Url};
use state::{RequestResponseState, RequestTask, SendRequestResponse};
use tokio::time::Instant;

use crate::{
    app_project::models::{self, RequestModel, ResponseModel, SendRequest, SendRequestKey},
    programs::tui::{
        common::component::{Drawable, Interactive},
        config::{keybinding, Config},
        elements::Separator,
        event_handler::{AppMessage, EventSender, Events},
        pane::text_editor::TextEditor,
    },
};

use super::{action::PaneAction, ElementFocus};

mod binary_viewer;
mod body_viewer;
mod headers_table;
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
                locked.headers_table.replace(response.headers.as_slice());
                let mime_type = response
                    .headers
                    .iter()
                    .find(|(k, _v)| k == CONTENT_TYPE.as_str())
                    .and_then(|(_k, v)| v.parse::<mime::Mime>().ok());
                locked.body_viewer = response_into_body_content(&response.body, &mime_type);
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

        let send_req_task = send_request(
            req,
            (Arc::clone(&send_req), id.clone()),
            Arc::clone(&self.response_content),
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
            Constraint::Fill(50),
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

                    let response_content = Arc::clone(&self.response_content);

                    match self.tab {
                        Tab::Headers => {
                            painter.render(move |frame| {
                                let locked = response_content.read().unwrap();
                                let table_ui = locked.headers_table.table_ui(&self.config.theme);
                                frame.render_widget(table_ui, self.content_area);
                            });
                        }
                        Tab::Response => {
                            painter.render(move |frame| {
                                let locked = response_content.read().unwrap();
                                frame.render_widget(&locked.body_viewer, self.content_area);
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

// pub enum ResponseViewerEffect {
//     NextFocus,
//     PreviousFocus,
// }

fn send_request(
    req: &RequestModel,
    (state, request_key): (SendRequestResponse, SendRequestKey),
    response_content: Arc<RwLock<ResponseContent>>,
    sender: EventSender,
) -> RequestTask {
    let (tx, tr) = tokio::sync::oneshot::channel();

    let client_builder = request_into_builder(req);

    let jh = tokio::spawn(async move {
        let timer = Instant::now();

        tokio::select! {
            result = client_builder.send() => {
                match result {
                    Ok(res) => {
                        let key_value_headers: Vec<(String, String)> = res.headers().iter().map(|(k,v)| (k.to_string(), v.to_str().unwrap().into())).collect();
                        let content_type = res.headers().get(CONTENT_TYPE).and_then(|val| val.to_str().ok()).and_then(|val| val.parse::<Mime>().ok());
                        let response_status = res.status().as_u16();
                        let response_body = res.bytes().await.unwrap();


                        let response = ResponseModel { duration:timer.elapsed(), status: response_status, body: response_body.to_vec(), headers: key_value_headers.clone() };
                        *state.write().unwrap() = SendRequest::Finish(response);

                        if response_content.read().unwrap().request_key.as_ref().map(|key| *key != request_key).unwrap_or(false) {
                            // So the current visual UI request is diferent from the local request executing
                            return;
                        }

                        {
                            // Mutate the headers state
                            let mut locked = response_content.write().unwrap();
                            locked.headers_table.replace(key_value_headers);
                        };

                        // set body content
                        let mut locked = response_content.write().unwrap();
                        locked.body_viewer = response_into_body_content(&response_body, &content_type);
                    },
                    Err(_e) => {},
                }
            }
            _ = tr => {
            }

        }

        let _ = sender.send(AppMessage::Draw).await;
    });

    RequestTask::new(tx, jh)
}

fn request_into_builder(req: &RequestModel) -> RequestBuilder {
    let url = Url::parse(req.url()).unwrap();
    let method = match req.method() {
        models::HttpMethod::Options => reqwest::Method::OPTIONS,
        models::HttpMethod::Get => reqwest::Method::GET,
        models::HttpMethod::Post => reqwest::Method::POST,
        models::HttpMethod::Put => reqwest::Method::PUT,
        models::HttpMethod::Delete => reqwest::Method::DELETE,
        models::HttpMethod::Head => reqwest::Method::HEAD,
        models::HttpMethod::Patch => reqwest::Method::PATCH,
    };

    let mut client_builder = ClientBuilder::new()
        .referer(false)
        .build()
        .unwrap()
        .request(method, url);

    for (key, value) in req.headers_map() {
        client_builder = client_builder.header(key, value);
    }

    client_builder
}

fn response_into_body_content(bytes: &[u8], content_type: &Option<Mime>) -> BodyContentView {
    if let Some(mime_type) = content_type {
        let is_text_based = match (mime_type.type_(), mime_type.subtype()) {
            (mime::TEXT, _) => true,
            (mime::APPLICATION, sub) => {
                matches!(
                    sub.as_str(),
                    "json" | "xml" | "xhtml+xml" | "x-www-form-urlencoded"
                )
            }
            _ => false,
        };

        if is_text_based {
            let encoding_name = mime_type
                .get_param("charset")
                .map(|charset| charset.as_str())
                .unwrap_or("utf-8");

            let encoding = Encoding::for_label(encoding_name.as_bytes()).unwrap_or(UTF_8);

            let (text, _, _) = encoding.decode(bytes);

            let mut text_editor = TextEditor::new(false);

            text_editor.insert_str(text.as_ref());

            return BodyContentView::Text(text_editor);
        }
    }

    // Fallback: show the content as hexdump
    let dump_viewer = BinaryViewer::from_bytes(bytes);
    BodyContentView::Binary(dump_viewer)
}
