use std::{collections::HashMap, time::Duration};

use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Tabs},
    Frame,
};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

use crate::{
    programs::tui_v2::{
        app_event::{AppTask, TaskSender},
        common::{
            elements::{draw_center_span, ui_block, ui_highlight},
            table_grid::UiTableGrid,
        },
        events::DrawSignal,
    },
    store::models::{HttpMethod, TimeId},
};

use super::{
    common::params_table::readonly_params,
    state_v2::{
        collections::{BodyContent, RequestItem},
        responses::{ResponseBody, ResponseStatus},
        table::{TableRow, TableState},
        PaneState, SectionFocus,
    },
};

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
        pane_id: TimeId,
        req_itm: &RequestItem,
        task_sender: TaskSender,
    ) {
        let Some(req) = from_req_state(req_itm) else {
            return;
        };
        let req_id = req_itm.id().to_string();

        let _ = task_sender
            .send(AppTask::for_request(pane_id, req_id, req))
            .await;
    }

    // pub fn cancel_req(&mut self, req_id: String, st: &Responses) {
    //     // we need abort the previous task
    //     if let Some(t) = self.tasks.remove(&req_id) {
    //         t.abort();
    //     }

    //     st.map
    //         .write()
    //         .unwrap()
    //         .insert(req_id, super::state::ResponseStatus::Cancelled);
    // }
}

impl ResponsesViewer {
    pub fn draw(&self, area: Rect, frame: &mut Frame, state: &PaneState) {
        let Some(req_id) = state.collections.current_req().map(|r| r.id().to_string()) else {
            return;
        };

        let is_focus = state.focus == SectionFocus::ResponseViewer;
        let block = ui_block("", is_focus);

        let inner_area = block.inner(area);
        let (line_area, tabs_area, content_area) = {
            let [line_status, tabs, content] = Layout::vertical([
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Fill(1),
            ])
            .areas(inner_area);
            (line_status, tabs, content)
        };

        match state.responses.list.get(&req_id) {
            Some(response) => {
                match response {
                    ResponseStatus::Fetching => {
                        draw_center_span("Sending...".blue(), area, frame);
                    }
                    ResponseStatus::Error(st) => {
                        draw_center_span(st.as_str().red(), area, frame);
                    }
                    ResponseStatus::Cancelled => {
                        draw_center_span("Request cancelled".into(), area, frame);
                    }
                    ResponseStatus::Success(res) => {
                        //- Render line
                        let line = Line::from_iter([
                            Span::raw(&res.version),
                            Span::raw("   "),
                            Span::raw(res.status.to_string()),
                            Span::raw(format!("{}", res.status_text)),
                            Span::raw("   "),
                            Span::raw(format!("{}", duration_as_str(res.duration))),
                            Span::raw("   "),
                            Span::raw(format!("{}", res.content_type)),
                        ]);
                        frame.render_widget(line, line_area);

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
                                .border_style(Style::default().fg(
                                    is_focus.then_some(Color::Blue).unwrap_or(Color::DarkGray),
                                )),
                        );
                        frame.render_widget(tabs, tabs_area);

                        match self.section.tab {
                            SectionTab::Headers => {
                                readonly_params(&res.headers).draw(content_area, frame);
                            }
                            SectionTab::Body => match &res.body {
                                ResponseBody::Text(s) => {
                                    frame.render_widget(ratatui::text::Text::raw(s), content_area);
                                }
                                ResponseBody::Binary(_) => {}
                                ResponseBody::Empty => {
                                    draw_center_span("Empty body O_O.".into(), content_area, frame);
                                }
                            },
                            SectionTab::Cookie => {
                                UiTableGrid::new(
                                    [
                                        "Name".blue(),
                                        "Value".blue(),
                                        "Domain".blue(),
                                        "Path".blue(),
                                    ],
                                    [0.25, 0.25, 0.25, 0.25],
                                )
                                .with_rows(
                                    res.cookies
                                        .items()
                                        .iter()
                                        .map(|cookie| {
                                            [
                                                Span::raw(&cookie.name),
                                                Span::raw(&cookie.value),
                                                Span::raw(match cookie.domain {
                                                    Some(ref domain) => domain.clone(),
                                                    None => "-".to_string(),
                                                }),
                                                Span::raw(match cookie.path {
                                                    Some(ref path) => path.clone(),
                                                    None => "-".to_string(),
                                                }),
                                            ]
                                        })
                                        .collect(),
                                )
                                .with_index(
                                    res.cookies.idx().as_cell(),
                                    Style::default().on_dark_gray(),
                                )
                                .draw(content_area, frame);
                            }
                        }
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
                    ResponseStatus::Error(_) => todo!(),
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

fn from_req_state(req_state: &RequestItem) -> Option<reqwest::RequestBuilder> {
    let method = match req_state.method {
        HttpMethod::Options => reqwest::Method::OPTIONS,
        HttpMethod::Get => reqwest::Method::GET,
        HttpMethod::Post => reqwest::Method::POST,
        HttpMethod::Put => reqwest::Method::PUT,
        HttpMethod::Delete => reqwest::Method::DELETE,
        HttpMethod::Head => reqwest::Method::HEAD,
        HttpMethod::Patch => reqwest::Method::PATCH,
        HttpMethod::Custom(ref t) => reqwest::Method::from_bytes(t.as_bytes()).unwrap(),
    };

    let Ok(url) = reqwest::Url::parse(&req_state.url) else {
        return None;
    };

    let mut headers = HeaderMap::new();
    for header in req_state.headers.items().iter().filter(|i| i.enable) {
        let name = HeaderName::from_bytes(header.key.as_bytes()).unwrap();
        let value = HeaderValue::from_str(&header.value).unwrap();
        headers.insert(name, value);
    }

    let mut builder = reqwest::Client::new().request(method, url).headers(headers);

    builder = match &req_state.body {
        BodyContent::None => builder.body(reqwest::Body::default()),
        BodyContent::Text(js) => builder.body(reqwest::Body::wrap(js.clone())),
        BodyContent::FormUrlEncoded(table) => {
            let mut map = HashMap::new();
            for itm in table.items().iter().filter(|itm| itm.enable) {
                map.insert(itm.key.clone(), itm.value.clone());
            }

            builder.form(&map)
        }
        BodyContent::FormData(_) => builder,
        BodyContent::File(_) => builder,
    };

    Some(builder)
}

fn duration_as_str(d: Duration) -> String {
    let millis = d.as_millis();
    let secs = d.as_secs_f64();

    if secs < 1_f64 {
        return format!("{:.3}ms", millis);
    }

    let minutes = secs / 60_f64;

    if minutes < 1_f64 {
        return format!("{:.3}s", secs);
    }

    return format!("{:.4}m", minutes);
}
