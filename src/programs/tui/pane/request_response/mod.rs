use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex, RwLock},
};

use body_viewer::{BodyContentView, HexDumpViewer};
use crossterm::event::{KeyCode, KeyEventKind};
use headers_table::HeadersTable;
use ratatui::{
    layout::{Constraint, Layout, Margin, Rect},
    style::{Style, Stylize},
    text::Span,
    widgets::{Block, Borders, Tabs},
};
use reqwest::{header::CONTENT_TYPE, ClientBuilder, Url};
use state::{RequestResponseState, RequestTask, SendRequestResponse};
use tokio::time::Instant;

use crate::{
    programs::tui::{
        common::component::{Drawable, Interactive},
        events::EventSender,
        pane::text_editor::TextEditor,
    },
    store::models::{RequestModel, ResponseModel, SendRequest, SendRequestId},
};

use super::state::ElementFocus;

mod body_viewer;
mod headers_table;
mod state;

#[derive(Clone, Copy)]
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

pub struct ResponseViewerComponent {
    state: RequestResponseState,
    tab: Tab,
    headers_viewer: Arc<Mutex<HeadersTable>>,
    body_viewer: Arc<Mutex<BodyContentView>>,
    render_area: Rect,
    header_area: Rect,
    content_area: Rect,
}

impl ResponseViewerComponent {
    pub fn new() -> Self {
        Self {
            state: RequestResponseState::new(),
            tab: Tab::Response,
            headers_viewer: Arc::new(Mutex::new(HeadersTable::new(vec![]))),
            body_viewer: Arc::new(Mutex::new(BodyContentView::Empty)),
            render_area: Rect::default(),
            header_area: Rect::default(),
            content_area: Rect::default(),
        }
    }

    pub fn execute_req(&mut self, id: SendRequestId, req: &RequestModel, sender: EventSender) {
        let send_req = match self.state.get(&id) {
            Some(req) => {
                *req.write().unwrap() = SendRequest::Pending;
                req
            }
            None => Arc::new(RwLock::new(SendRequest::Pending)),
        };

        let send_req_task = send_request(
            req,
            Arc::clone(&send_req),
            (
                Arc::clone(&self.headers_viewer),
                Arc::clone(&self.body_viewer),
            ),
            sender,
        );

        self.state.add(id.clone(), send_req, send_req_task);
        self.state.set_current_response(Some(id));
    }
}

impl Drawable for ResponseViewerComponent {
    type Params = ElementFocus;

    fn set_area(&mut self, area: Rect) {
        let main_areas = Layout::vertical([Constraint::Length(2), Constraint::Fill(50)])
            .split(area.inner(Margin::new(1, 1)));

        self.render_area = area;
        self.header_area = main_areas[0];
        self.content_area = main_areas[1];
    }

    fn draw<'a: 'painter, 'painter>(
        &'a self,
        painter: &mut crate::programs::tui::common::component::Painter<'painter>,
        params: Self::Params,
    ) {
        match self
            .state
            .current_response()
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
            Some(response) => match &*response.read().unwrap() {
                crate::store::models::SendRequest::Pending => {
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
                crate::store::models::SendRequest::Finish(_) => {
                    painter.render(move |frame| {
                        let style = if params == super::state::ElementFocus::ResponseViewer {
                            Style::default().blue()
                        } else {
                            Style::default()
                        };

                        let block = Block::bordered().border_style(style);
                        frame.render_widget(block, self.render_area);

                        let tabs = Tabs::new([
                            format!(" {} ", Tab::Response.as_ref()),
                            format!(" {} ", Tab::Headers.as_ref()),
                        ])
                        .select(self.tab.as_idx())
                        .block(Block::new().borders(Borders::BOTTOM).border_style(style))
                        .highlight_style(Style::default().blue());

                        frame.render_widget(tabs, self.header_area);
                    });

                    match self.tab {
                        Tab::Headers => {
                            painter.render(|frame| {
                                let headers_table = self.headers_viewer.lock().unwrap();
                                let table_ui = headers_table.table_ui();
                                frame.render_widget(table_ui, self.content_area);
                            });
                        }
                        Tab::Response => {
                            painter.render(|frame| {
                                let body_viewer = self.body_viewer.lock().unwrap();
                                frame.render_widget(&*body_viewer, self.content_area);
                            });
                        }
                    }
                }
            },
        }
    }
}

impl Interactive for ResponseViewerComponent {
    type Effect = ResponseViewerEffect;

    fn on_key(&mut self, key: crossterm::event::KeyEvent) -> Option<Self::Effect> {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Tab => match self.tab {
                    Tab::Headers => return Some(ResponseViewerEffect::NextFocus),
                    Tab::Response => self.tab = Tab::Headers,
                },
                KeyCode::BackTab => match self.tab {
                    Tab::Headers => self.tab = Tab::Response,
                    Tab::Response => return Some(ResponseViewerEffect::PreviousFocus),
                },
                _ => match self.tab {
                    Tab::Headers => {
                        let mut headers_editor = self.headers_viewer.lock().unwrap();
                        match key.code {
                            KeyCode::Left | KeyCode::Char('h') => {
                                headers_editor.move_col_idx(false)
                            }
                            KeyCode::Right | KeyCode::Char('l') => {
                                headers_editor.move_col_idx(true)
                            }
                            KeyCode::Up | KeyCode::Char('k') => headers_editor.move_row_idx(false),
                            KeyCode::Down | KeyCode::Char('j') => headers_editor.move_row_idx(true),
                            _ => {}
                        }
                    }
                    Tab::Response => {
                        let mut body_viewer = self.body_viewer.lock().unwrap();
                        match &mut *body_viewer {
                            BodyContentView::Text(text_editor) => {
                                text_editor.handle_key(key);
                            }
                            BodyContentView::Binary(_hex_dump_viewer) => {}
                            BodyContentView::Empty => {}
                        }
                    }
                },
            }
        }

        None
    }
}

pub enum ResponseViewerEffect {
    NextFocus,
    PreviousFocus,
}

fn send_request(
    req: &RequestModel,
    state: SendRequestResponse,
    (table_headers, body_viewer): (Arc<Mutex<HeadersTable>>, Arc<Mutex<BodyContentView>>),
    sender: EventSender,
) -> RequestTask {
    let (tx, tr) = tokio::sync::oneshot::channel();

    let url = Url::parse(req.url()).unwrap();
    let method = match req.method() {
        crate::store::models::HttpMethod::Options => reqwest::Method::OPTIONS,
        crate::store::models::HttpMethod::Get => reqwest::Method::GET,
        crate::store::models::HttpMethod::Post => reqwest::Method::POST,
        crate::store::models::HttpMethod::Put => reqwest::Method::PUT,
        crate::store::models::HttpMethod::Delete => reqwest::Method::DELETE,
        crate::store::models::HttpMethod::Head => reqwest::Method::HEAD,
        crate::store::models::HttpMethod::Patch => reqwest::Method::PATCH,
    };

    let mut client_builder = ClientBuilder::new()
        .referer(false)
        .build()
        .unwrap()
        .request(method, url);

    for (key, value) in req.headers_map() {
        client_builder = client_builder.header(key, value);
    }

    let jh = tokio::spawn(async move {
        let timer = Instant::now();

        tokio::select! {
            result = client_builder.send() => {
                match result {
                    Ok(res) => {
                        let response = ResponseModel { duration:timer.elapsed(), status:res.status().as_u16(), body: "BODY", headers: HashMap::new() };
                        *state.write().unwrap() = SendRequest::Finish(response);

                        {
                            // Mutate the headers state
                            let key_value_headers: Vec<(String, String)> = res.headers().iter().map(|(k,v)| (k.to_string(), v.to_str().unwrap().into())).collect();
                            let mut headers_state = table_headers.lock().unwrap();
                            headers_state.replace(key_value_headers);
                        };

                        // Mutate the response body content state
                        let content_type = res.headers().get(CONTENT_TYPE).and_then(|val| val.to_str().ok());
                        match content_type {
                            Some(_content_type) => {
                                if _content_type.starts_with("text/") {
                                    let text = res.text().await.unwrap();
                                    let mut _body_viewer = body_viewer.lock().unwrap();
                                    let mut text_editor = TextEditor::new(false);
                                    text_editor.insert_str(&text);
                                    *_body_viewer = BodyContentView::Text(text_editor);
                                } else {
                                    let valid_text = HashSet::from(["application/json", "application/xml", "application/javascript", "application/x-www-form-urlencoded", "application/xhtml+xml"]);

                                    if valid_text.iter().any(|txt| valid_text.contains(txt)) {
                                        let text = res.text().await.unwrap();
                                        let mut _body_viewer = body_viewer.lock().unwrap();
                                        let mut text_editor = TextEditor::new(false);
                                        text_editor.insert_str(&text);
                                        *_body_viewer = BodyContentView::Text(text_editor);
                                    } else {
                                        let bytes_vec = res.bytes().await.unwrap().to_vec();
                                        let dump_viewer = HexDumpViewer::new(&bytes_vec);
                                        let mut _body_viewer = body_viewer.lock().unwrap();
                                        *_body_viewer = BodyContentView::Binary(dump_viewer);
                                    }


                                }
                            },
                            None => {
                                let bytes_vec = res.bytes().await.unwrap().to_vec();
                                let dump_viewer = HexDumpViewer::new(&bytes_vec);
                                let mut _body_viewer = body_viewer.lock().unwrap();
                                *_body_viewer = BodyContentView::Binary(dump_viewer);
                            }
                        };
                    },
                    Err(_e) => {},
                }
            }
            _ = tr => {
            }

        }

        let _ = sender.send(crate::programs::tui::events::Event::Draw);
    });

    RequestTask::new(tx, jh)
}
