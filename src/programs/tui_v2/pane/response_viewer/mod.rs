use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Span,
    widgets::{Block, BorderType, Borders, Tabs},
    Frame,
};
use reqwest::header::{HeaderName, HeaderValue};

use crate::{
    http::request,
    programs::tui_v2::{
        app::TaskGroupKey,
        common::{
            elements::{draw_center_span, ui_block, ui_highlight},
            table_grid::UiTableGrid,
        },
        events::DrawSignal,
        pane::{
            response_viewer::status_line::draw_line,
            state_v2::responses::{Body, Response},
        },
    },
    store::models::HttpMethod,
};

use super::{
    common::params_table::readonly_params,
    state_v2::{
        collections::{BodyContent, RequestItem},
        responses::{Cookie, ResponseBody, ResponseStatus},
        table::{TableRow, TableState},
        PaneState, SectionFocus,
    },
};

mod status_line;

enum SectionTab {
    Headers,
    Body,
    Cookie,
}

impl SectionTab {
    pub fn label(&self) -> &'static str {
        match self {
            SectionTab::Headers => "Headers",
            SectionTab::Body => "Body",
            SectionTab::Cookie => "Cookie",
        }
    }
}

struct Section {
    tab: SectionTab,
}

impl Section {
    pub fn new() -> Self {
        Self {
            tab: SectionTab::Body,
        }
    }

    pub fn next(&mut self) {
        self.tab = match self.tab {
            SectionTab::Body => SectionTab::Headers,
            SectionTab::Headers => SectionTab::Cookie,
            SectionTab::Cookie => SectionTab::Body,
        };
    }

    pub fn previous(&mut self) {
        self.tab = match self.tab {
            SectionTab::Headers => SectionTab::Body,
            SectionTab::Body => SectionTab::Cookie,
            SectionTab::Cookie => SectionTab::Headers,
        };
    }

    pub fn idx(&self) -> u8 {
        match self.tab {
            SectionTab::Body => 0,
            SectionTab::Headers => 1,
            SectionTab::Cookie => 2,
        }
    }
}

pub struct ResponsesViewer {
    section: Section,
    _draw_signal: DrawSignal,
}

impl ResponsesViewer {
    pub fn new(draw_signal: DrawSignal) -> Self {
        Self {
            section: Section::new(),
            _draw_signal: draw_signal,
        }
    }

    pub async fn send_req_v2(
        &mut self,
        req_itm: &RequestItem,
        task_sender: &crate::programs::tui_v2::task::TaskSender<TaskGroupKey>,
    ) {
        let Some(req) = to_http_req(req_itm) else {
            return;
        };
        let req_id = req_itm.id().to_string();

        let _ = task_sender
            .send(crate::programs::tui_v2::task::TaskType::HttpRequest { id: req_id, req })
            .await;
    }
}

impl ResponsesViewer {
    fn draw_response(&self, res: &Response, area: Rect, frame: &mut Frame, is_focus: bool) {
        let (line_area, tabs_area, content_area) = {
            let [line_status, tabs, content] = Layout::vertical([
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Fill(1),
            ])
            .areas(area);
            (line_status, tabs, content)
        };

        draw_line(res, line_area, frame);

        //- Render tabs
        let tabs = Tabs::new([
            format!("{}", SectionTab::Body.label()),
            format!(
                "{} ({})",
                SectionTab::Headers.label(),
                res.headers.len() + res.cookies.len()
            ),
            format!("{} ({})", SectionTab::Cookie.label(), res.cookies.len()),
        ])
        .select(self.section.idx() as usize)
        .highlight_style(ui_highlight())
        .block(
            Block::bordered()
                .border_type(if is_focus {
                    BorderType::Thick
                } else {
                    BorderType::Plain
                })
                .borders(Borders::BOTTOM | Borders::TOP)
                .border_style(
                    Style::default().fg(is_focus.then_some(Color::Blue).unwrap_or(Color::DarkGray)),
                ),
        );
        frame.render_widget(tabs, tabs_area);

        match self.section.tab {
            SectionTab::Headers => {
                readonly_params(&res.headers).draw(content_area, frame);
            }
            SectionTab::Body => match &res.body {
                ResponseBody::InMemory(body) => match body {
                    Body::Text(s) => {
                        frame.render_widget(ratatui::text::Text::raw(s), content_area);
                    }
                    Body::Binary(_) => {}
                },
                ResponseBody::OnDisk(path_buf) => {
                    draw_center_span(
                        format!("Content stored in {:?}", path_buf).into(),
                        area,
                        frame,
                    );
                }
            },
            SectionTab::Cookie => {
                cookie_table(&res.cookies).draw(content_area, frame);
            }
        }
    }

    pub fn draw(&self, area: Rect, frame: &mut Frame, state: &PaneState) {
        let Some(req_id) = state.collections.current_req().map(|r| r.id().to_string()) else {
            return;
        };

        let is_focus = state.focus == SectionFocus::ResponseViewer;

        match state.responses.list.get(&req_id) {
            Some(response) => {
                let block = ui_block("", is_focus);
                match response {
                    ResponseStatus::Fetching => {
                        draw_center_span("Sending...".blue(), area, frame);
                    }
                    ResponseStatus::Error { title, .. } => {
                        draw_center_span(title.as_str().red(), area, frame);
                    }
                    ResponseStatus::Cancelled => {
                        draw_center_span("Request cancelled".into(), area, frame);
                    }
                    ResponseStatus::Success(res) => {
                        self.draw_response(res, block.inner(area), frame, is_focus);
                    }
                }

                frame.render_widget(block, area);
            }
            None => draw_center_span("Send request with SHIT + S".into(), area, frame),
        }
    }

    pub fn draw_overlay(&self, _frame: &mut Frame) {}

    pub fn handle_key(&mut self, key: KeyEvent, state: &mut PaneState) {
        match key.code {
            crossterm::event::KeyCode::Left => {
                self.section.previous();
            }
            crossterm::event::KeyCode::Right => {
                self.section.next();
            }
            crossterm::event::KeyCode::Tab => {
                state.focus = SectionFocus::Collections;
            }
            crossterm::event::KeyCode::BackTab => {
                state.focus = SectionFocus::RequestBuilder;
            }
            _ => {
                let Some(res) = state
                    .collections
                    .current_req()
                    .map(|r| r.id().to_string())
                    .and_then(|id| state.responses.list.get_mut(&id))
                else {
                    return;
                };

                match res {
                    ResponseStatus::Success(response) => match self.section.tab {
                        SectionTab::Headers => {
                            self.handle_table_basic_keys(key, &mut response.headers);
                        }
                        SectionTab::Body => {}
                        SectionTab::Cookie => {
                            self.handle_table_basic_keys(key, &mut response.cookies);
                        }
                    },
                    ResponseStatus::Error { .. } => todo!(),
                    _ => {}
                }
            }
        }
    }

    fn handle_table_basic_keys<R: TableRow>(&mut self, key: KeyEvent, table: &mut TableState<R>) {
        match key.code {
            crossterm::event::KeyCode::Char(ch) => match ch {
                'j' => table.next_row(),
                'k' => table.prev_row(),
                'h' => table.prev_cell(),
                'l' => table.next_cell(),
                _ => {}
            },
            _ => {}
        }
    }
}

fn to_http_req(req_item: &RequestItem) -> Option<request::HttpRequest> {
    let method = match req_item.method {
        HttpMethod::Options => reqwest::Method::OPTIONS,
        HttpMethod::Get => reqwest::Method::GET,
        HttpMethod::Post => reqwest::Method::POST,
        HttpMethod::Put => reqwest::Method::PUT,
        HttpMethod::Delete => reqwest::Method::DELETE,
        HttpMethod::Head => reqwest::Method::HEAD,
        HttpMethod::Patch => reqwest::Method::PATCH,
        HttpMethod::Custom(ref t) => reqwest::Method::from_bytes(t.as_bytes()).unwrap(),
    };

    let Ok(url) = reqwest::Url::parse(&req_item.url) else {
        return None;
    };

    let mut req = request::HttpRequest::new(method, url);

    for header in req_item.headers.items().iter().filter(|i| i.enable) {
        let name = HeaderName::from_bytes(header.key.as_bytes()).unwrap();
        let value = HeaderValue::from_str(&header.value).unwrap();
        req.add_header(name, value);
    }

    match &req_item.body {
        BodyContent::Text(txt) => {
            req.set_body_text(Vec::from(txt.as_bytes()));
        }
        BodyContent::File(file_info) => match file_info {
            super::state_v2::collections::FileContent::None => {}
            super::state_v2::collections::FileContent::Content { path, .. } => {
                req.set_body_file(path.clone());
            }
        },
        _ => {}
    };

    Some(req)
}

fn check_bool_as_txt(flag: bool, txt: &str) -> &str {
    if flag {
        txt
    } else {
        "-"
    }
}

fn cookie_table(table: &TableState<Cookie>) -> UiTableGrid<'_, '_, 7> {
    let flags = |cookie: &Cookie| {
        format!(
            "{} {} {}",
            check_bool_as_txt(cookie.http_only, "H"),
            check_bool_as_txt(cookie.secure, "S"),
            check_bool_as_txt(cookie.partitioned, "P")
        )
    };

    UiTableGrid::new(
        [
            "Name".blue(),            // 15
            "Value".blue(),           // 15
            "Domain".blue(),          // 15
            "Path".blue(),            // 10
            "Expires/Max-Age".blue(), // 25
            "SameSite".blue(),        // 10
            "H|S|P".blue(),           // 10
        ],
        [0.15, 0.15, 0.15, 0.10, 0.25, 0.10, 0.10],
    )
    .with_rows(
        table
            .items()
            .iter()
            .map(|cookie| {
                [
                    Span::raw(&cookie.name),
                    Span::raw(&cookie.value),
                    Span::raw(
                        cookie
                            .domain
                            .as_ref()
                            .map_or("-".to_string(), |d| d.clone()),
                    ),
                    Span::raw(match cookie.path {
                        Some(ref path) => path.clone(),
                        None => "-".to_string(),
                    }),
                    Span::raw(cookie.expires.as_ref().map_or_else(
                        || cookie.max_ge.as_ref().map_or("".to_string(), |e| e.clone()),
                        |e| e.clone(),
                    )),
                    Span::raw(
                        cookie
                            .same_site
                            .as_ref()
                            .map_or("-".to_string(), |e| e.clone()),
                    ),
                    Span::raw(flags(cookie)),
                ]
            })
            .collect(),
    )
    .with_index(table.idx().as_cell(), Style::default().on_dark_gray())
}
