use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
};

use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout, Margin, Rect},
    style::{Style, Stylize},
    text::Span,
    widgets::{Block, Borders, Tabs},
};
use reqwest::{ClientBuilder, Url};
use state::{RequestResponseState, RequestTask, SendRequestResponse};
use tokio::time::Instant;

use crate::{
    programs::tui::{
        common::component::{Drawable, Interactive},
        events::EventSender,
    },
    store::models::{RequestModel, ResponseModel, SendRequest, SendRequestId},
};

use super::{
    params_table::{TableParams, TableParamsUi},
    state::ElementFocus,
};

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
    headers_editor: Arc<Mutex<TableParams>>,
    render_area: Rect,
    header_area: Rect,
    content_area: Rect,
}

impl ResponseViewerComponent {
    pub fn new() -> Self {
        Self {
            state: RequestResponseState::new(),
            tab: Tab::Response,
            headers_editor: Arc::new(Mutex::new(TableParams::new())),
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

        let send_req_task = send_request(req, Arc::clone(&send_req), sender);

        self.state.add(id.clone(), send_req, send_req_task);
        self.state.set_current_response(Some(id));
    }

    // pub fn delete_response() {}
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
            Some(response) => {
                match &*response.read().unwrap() {
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
                                // painter.render(|frame| {
                                //     self.headers_editor.lock().unwrap().draw(&mut painter, ());
                                // });
                                // painter.render(|frame| {
                                //     let table = TableParamsUi::new(Vec::new());
                                //     frame.render_widget(table, self.content_area);
                                // });
                            }
                            Tab::Response => {
                                // self.body_editor.draw(painter, state);
                            }
                        }
                    }
                }
            }
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
                        let headers_editor = &mut *self.headers_editor.lock().unwrap();
                        headers_editor.on_key(key);
                    }
                    Tab::Response => {
                        // self.body_editor.on_key(key, mutator, state);
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
