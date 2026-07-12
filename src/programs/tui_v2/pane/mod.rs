use crate::{
    http::{response::HttpResponse, Error as HttpError},
    programs::tui_v2::common::text_editor::TextEditor,
    store::models::{ProjectModel, TimeId},
};
use actions::EffectsCollector;
use collections::CollectionsSidebar;
use crossterm::event::{KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    Frame,
};
use request_bar::RequestBar;
use request_builder::RequestBuilder;
use response_viewer::ResponsesViewer;
use state_v2::{
    collections::ListIdx,
    from_project_model,
    responses::{self, ResponseStatus},
    table, PaneState, SectionFocus,
};

use super::{app::TaskGroupKey, ctx::InitialCtx, task::TaskSender};
mod actions;
mod collections;
mod common;
mod request_bar;
mod request_builder;
mod response_viewer;
mod state_v2;

pub struct Pane {
    project_id: String,
    project_name: String,
    state: PaneState,
    collection_sidebar: CollectionsSidebar,
    request_bar: RequestBar,
    request_builder: RequestBuilder,
    response_viewer: ResponsesViewer,
    _task_sender: TaskSender<TaskGroupKey>,
}

type ProjectDetails = (String, String);

impl Pane {
    pub fn new(
        (project_id, project_name): ProjectDetails,
        state: PaneState,
        ctx: InitialCtx,
    ) -> Self {
        Self {
            project_id,
            project_name,
            state,
            collection_sidebar: CollectionsSidebar::new(),
            request_bar: RequestBar::new(),
            request_builder: RequestBuilder::new(ctx.draw_signal.clone()),
            response_viewer: ResponsesViewer::new(ctx.draw_signal),
            _task_sender: ctx.task_sender,
        }
    }

    pub fn name(&self) -> &str {
        &self.project_name
    }

    pub fn id(&self) -> &str {
        &self.project_id
    }
}

impl Pane {
    pub fn draw(&self, area: Rect, frame: &mut Frame) {
        let [sidebar_area, main_area] =
            Layout::horizontal([Constraint::Percentage(30), Constraint::Percentage(70)])
                .areas(area);

        self.collection_sidebar
            .draw(sidebar_area, frame, &self.state);

        match self.state.collections.idx {
            ListIdx::None | ListIdx::Group(_) => {}
            ListIdx::Item(_, _) => {
                let [bar_area, builder_area, response_area] = Layout::vertical([
                    Constraint::Length(3),
                    Constraint::Fill(1),
                    Constraint::Fill(1),
                ])
                .areas(main_area);
                self.request_bar.draw(bar_area, frame, &self.state);
                self.request_builder.draw(builder_area, frame, &self.state);
                self.response_viewer.draw(response_area, frame, &self.state);
            }
        }

        self.collection_sidebar
            .draw_overlay(area, frame, &self.state);
        self.request_bar.draw_overlay(area, frame);
        self.request_builder.draw_overlay(frame);
        self.response_viewer.draw_overlay(frame);
    }

    pub async fn handle_key(&mut self, key: KeyEvent) -> bool {
        if crossterm::event::KeyCode::Char('S') == key.code
            && key.modifiers.contains(KeyModifiers::SHIFT)
        {
            if let Some(req) = self.state.collections.current_req() {
                self.state
                    .responses
                    .list
                    .insert(req.id().to_string(), ResponseStatus::Fetching);

                self.response_viewer
                    .send_req_v2(req, &self._task_sender)
                    .await;
                return true;
            }
        }

        let mut effects = EffectsCollector::new();

        match self.state.focus {
            SectionFocus::Collections => {
                self.collection_sidebar
                    .handle_key(key, &mut self.state, &mut effects);
            }
            SectionFocus::RequestBar => {
                self.request_bar.handle_key(key, &mut self.state);
            }
            SectionFocus::RequestBuilder => {
                self.request_builder.handle_key(key, &mut self.state).await;
            }
            SectionFocus::ResponseViewer => {
                self.response_viewer.handle_key(key, &mut self.state);
            }
        }

        while let Some(effect) = effects.next() {
            match effect {
                actions::PaneActionEffect::ChangeIdx => {
                    self.request_bar.sync(&mut self.state);
                    self.request_builder.sync(&mut self.state);
                }
            }
        }

        true
    }

    pub fn handle_response_v2(&mut self, req_id: TimeId, response: HttpResponse) {
        let mut headers = table::TableState::new();

        for (key, value) in response.headers {
            headers.add_row(responses::ReadonlyHeader { key, value });
        }

        let mut cookies = table::TableState::new();

        for cookie in response.cookies {
            cookies.add_row(responses::Cookie {
                name: cookie.name,
                value: cookie.value,
                domain: cookie.domain,
                expires: cookie.expires,
                max_ge: cookie.max_age,
                path: cookie.path,
                http_only: cookie.http_only.unwrap_or(false),
                partitioned: cookie.partitioned.unwrap_or(false),
                secure: cookie.secure.unwrap_or(false),
                same_site: cookie.same_site,
            });
        }

        let response_body = match response.body {
            crate::http::response::HttpResBody::Contained(http_body_content) => {
                let body = match http_body_content {
                    crate::http::response::HttpBodyContent::Text(s) => {
                        let mut editor = TextEditor::new(false);
                        editor.insert_str(&s);
                        responses::Body::Text(editor)
                    }
                    crate::http::response::HttpBodyContent::Bytes(items) => {
                        responses::Body::Binary(items)
                    }
                };
                responses::ResponseBody::InMemory(body)
            }
            crate::http::response::HttpResBody::DiskFile(p) => responses::ResponseBody::OnDisk(p),
        };

        let response = responses::Response {
            status: response.status,
            status_text: response.status_text,
            version: response.version,
            duration: response.duration,
            size_bytes: response.body_bytes,
            content_type: response.content_type,
            headers,
            cookies,
            body: response_body,
        };

        self.state
            .responses
            .list
            .insert(req_id, responses::ResponseStatus::Success(response));
    }

    pub fn handle_http_error(&mut self, req_id: TimeId, error: HttpError) {
        self.state.responses.list.insert(
            req_id,
            responses::ResponseStatus::Error {
                title: error.title,
                desc: error.description,
            },
        );
    }
}

pub async fn new_pane(project: ProjectModel, ctx: InitialCtx) -> Pane {
    let project_parts = (project.id.clone(), project.name.clone());
    let pane_state = from_project_model(project).await;
    Pane::new(project_parts, pane_state, ctx)
}
