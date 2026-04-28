use std::collections::HashMap;

use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Span,
    widgets::{Block, BorderType, Borders, Tabs},
    Frame,
};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use tokio::task::JoinHandle;

use crate::{
    programs::tui_v2::{
        app_event::{AppTask, TaskSender},
        common::elements::{draw_center_span, ui_block, ui_highlight},
        events::DrawSignal,
    },
    store::models::{HttpMethod, TimeId},
};

use super::{
    common::params_table::params_to_ui,
    state::{BodyContent, PaneState, ParamsTable, RequestItem, Responses, SectionFocus},
};

enum SectionTab {
    Headers,
    Body,
}

impl SectionTab {
    pub fn label(&self) -> &'static str {
        match self {
            SectionTab::Headers => "Headers",
            SectionTab::Body => "Body",
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
            SectionTab::Headers => SectionTab::Body,
            SectionTab::Body => SectionTab::Headers,
        };
    }

    pub fn idx(&self) -> u8 {
        match self.tab {
            SectionTab::Body => 0,
            SectionTab::Headers => 1,
        }
    }
}

type Tasks = HashMap<TimeId, JoinHandle<()>>;

pub struct ResponsesViewer {
    section: Section,
    tasks: Tasks,
    _draw_signal: DrawSignal,
}

impl ResponsesViewer {
    pub fn new(draw_signal: DrawSignal) -> Self {
        Self {
            section: Section::new(),
            tasks: HashMap::new(),
            _draw_signal: draw_signal,
        }
    }

    pub async fn send_req_v2(&mut self, req_itm: &RequestItem, task_sender: TaskSender) {
        let Some(req) = from_req_state(req_itm) else {
            return;
        };
        let req_id = req_itm.id().to_string();

        let _ = task_sender.send(AppTask::Request(req_id, req)).await;
    }

    pub fn send_req(&mut self, req_itm: &RequestItem, st: &Responses) {
        let Some(req) = from_req_state(req_itm) else {
            return;
        };
        let req_id = req_itm.id().to_string();

        // we need abort the previous task
        if let Some(t) = self.tasks.remove(&req_id) {
            t.abort();
        }

        let map = st.map.clone();

        map.write()
            .unwrap()
            .insert(req_id.to_string(), super::state::ResponseStatus::Fetching);

        let draw_signal = self._draw_signal.clone();

        let jh = tokio::spawn(async move {
            let timer = tokio::time::Instant::now();
            match req.send().await {
                Ok(res) => {
                    let duration = timer.elapsed();
                    let mut headers = ParamsTable::new();

                    for (name, value) in res.headers() {
                        // NOTE: support non-ascii text?
                        let Ok(value) = value.to_str() else {
                            continue;
                        };

                        headers.add_item(super::state::ParamItem {
                            enable: true,
                            key: name.as_str().to_string(),
                            value: value.to_string(),
                        });
                    }

                    let response = super::state::Response {
                        status: res.status().as_u16(),
                        status_text: res.status().as_str().to_string(),
                        version: format!("{:?}", res.version()),
                        duration,
                        headers: headers,
                        body: super::state::ResponseBody::Empty,
                        size_bytes: res.bytes().await.map(|b| b.len()).unwrap_or(0),
                    };
                    map.write()
                        .unwrap()
                        .insert(req_id, super::state::ResponseStatus::Success(response));
                }
                Err(e) => {
                    map.write()
                        .unwrap()
                        .insert(req_id, super::state::ResponseStatus::Error(e.to_string()));
                }
            };

            draw_signal.draw();
        });

        self.tasks.insert(req_itm.id().to_string(), jh);
    }

    pub fn cancel_req(&mut self, req_id: String, st: &Responses) {
        // we need abort the previous task
        if let Some(t) = self.tasks.remove(&req_id) {
            t.abort();
        }

        st.map
            .write()
            .unwrap()
            .insert(req_id, super::state::ResponseStatus::Cancelled);
    }
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
                    super::state::ResponseStatus::Fetching => {
                        draw_center_span("Sending...".blue(), area, frame);
                    }
                    super::state::ResponseStatus::Error(st) => {
                        draw_center_span(st.as_str().red(), area, frame);
                    }
                    super::state::ResponseStatus::Cancelled => {
                        draw_center_span("Request cancelled".into(), area, frame);
                    }
                    super::state::ResponseStatus::Success(res) => {
                        //- Render line
                        frame.render_widget(Span::raw("Status line"), line_area);

                        //- Render tabs
                        let tabs = Tabs::new([
                            format!("{} ({})", SectionTab::Body.label(), 0),
                            format!("{} ({})", SectionTab::Headers.label(), 0),
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
                                params_to_ui(&res.headers).draw(content_area, frame);
                            }
                            SectionTab::Body => match &res.body {
                                super::state::ResponseBody::Text(s) => {
                                    frame.render_widget(ratatui::text::Text::raw(s), content_area);
                                }
                                super::state::ResponseBody::Binary(_) => {}
                                super::state::ResponseBody::Empty => {
                                    draw_center_span("Empty body O_O.".into(), content_area, frame);
                                }
                            },
                        }
                    }
                }

                frame.render_widget(block, area);
            }
            None => draw_center_span("Send request with SHIT + S".into(), area, frame),
        }
    }

    pub fn draw_overlay(&self, frame: &mut Frame) {}

    pub fn handle_key(&mut self, key: KeyEvent, state: &mut PaneState) {
        match key.code {
            crossterm::event::KeyCode::Left => {
                self.section.next();
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
                // if let Some(request) = state.collections.current_req_mut() {
                //     match self.section {
                //         Section::Headers => self.handle_table_key(key, &mut request.headers),
                //         Section::Params => self.handle_table_key(key, &mut request.params),
                //         Section::Body => self.handle_body_key(key, &mut request.body).await,
                //     }
                // }
            }
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
    for header in req_state.headers.items.iter().filter(|i| i.enable) {
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
            for itm in table.items.iter().filter(|itm| itm.enable) {
                map.insert(itm.key.clone(), itm.value.clone());
            }

            builder.form(&map)
        }
        BodyContent::FormData(_) => builder,
        BodyContent::File(_) => builder,
    };

    Some(builder)
}
