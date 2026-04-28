use std::os::unix::fs::MetadataExt;

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
use state::{
    BodyContent, BodyForm, CollectionItem, CollectionsList, Environments, FileInfo, PaneState,
    ParamsTable, RequestItem,
};

use crate::store::models::ProjectModel;

use super::{
    app_event::{ReqResponse, TaskSender},
    events::DrawSignal,
};
mod actions;
mod collections;
mod common;
mod request_bar;
mod request_builder;
mod response_viewer;
mod state;

pub struct Pane {
    project_id: String,
    project_name: String,
    state: PaneState,
    collection_sidebar: CollectionsSidebar,
    request_bar: RequestBar,
    request_builder: RequestBuilder,
    response_viewer: ResponsesViewer,
    _task_sender: TaskSender,
}

type ProjectDetails = (String, String);

impl Pane {
    pub fn from_project(
        project: ProjectModel,
        draw_signal: DrawSignal,
        task_sender: TaskSender,
    ) -> Self {
        Self::new(
            (project.id, project.name),
            PaneState::from_parts(
                project.collections,
                project.environments,
                project.selected_env_context,
            ),
            draw_signal,
            task_sender,
        )
    }

    pub fn new(
        (project_id, project_name): ProjectDetails,
        state: PaneState,
        draw_signal: DrawSignal,
        task_sender: TaskSender,
    ) -> Self {
        Self {
            project_id,
            project_name,
            state,
            collection_sidebar: CollectionsSidebar::new(),
            request_bar: RequestBar::new(),
            request_builder: RequestBuilder::new(draw_signal.clone()),
            response_viewer: ResponsesViewer::new(draw_signal),
            _task_sender: task_sender,
        }
    }

    pub fn name(&self) -> &str {
        &self.project_name
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
            state::ListIdx::None | state::ListIdx::Group(_) => {}
            state::ListIdx::Item(_, _) => {
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
                    .insert(req.id().to_string(), state::ResponseStatus::Fetching);

                self.response_viewer
                    .send_req_v2(req, self._task_sender.clone())
                    .await;
                return true;
            }
        }

        let mut effects = EffectsCollector::new();

        match self.state.focus {
            state::SectionFocus::Collections => {
                self.collection_sidebar
                    .handle_key(key, &mut self.state, &mut effects);
            }
            state::SectionFocus::RequestBar => {
                self.request_bar.handle_key(key, &mut self.state);
            }
            state::SectionFocus::RequestBuilder => {
                self.request_builder.handle_key(key, &mut self.state).await;
            }
            state::SectionFocus::ResponseViewer => {
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

    pub fn handle_response(&mut self, req_res: ReqResponse) {
        let list = &mut self.state.responses.list;

        match req_res {
            ReqResponse::Err(req_id, txt) => {
                list.insert(req_id, state::ResponseStatus::Error(txt));
            }
            ReqResponse::Sucess(req_id, res) => {
                let response = state::Response {
                    status: res.status,
                    status_text: res.status_text,
                    version: res.version,
                    duration: res.duration,
                    size_bytes: 10,
                    headers: ParamsTable::new(),
                    body: state::ResponseBody::Empty,
                };
                list.insert(req_id, state::ResponseStatus::Success(response));
            }
        }
    }
}

pub async fn new_pane(
    project: ProjectModel,
    draw_signal: DrawSignal,
    task_sender: TaskSender,
) -> Pane {
    let mut list = Vec::new();

    for coll in project.collections {
        let mut reqs = Vec::new();

        for req in coll.requests {
            let body = match req.body {
                crate::store::models::RequestBody::None => BodyContent::None,
                crate::store::models::RequestBody::Text(st) => BodyContent::Text(st),
                crate::store::models::RequestBody::Json(v) => BodyContent::Text(v.to_string()),
                crate::store::models::RequestBody::FormUrlEncoded(m) => {
                    BodyContent::FormUrlEncoded(ParamsTable::from_model(m))
                }
                crate::store::models::RequestBody::FormData(m) => {
                    BodyContent::FormData(BodyForm::from_model(m))
                }
                crate::store::models::RequestBody::File(path) => {
                    if path.is_file() {
                        match tokio::fs::metadata(&path).await {
                            Ok(m) => {
                                let name = path.file_name().unwrap().to_str().unwrap().to_string();
                                let path_str = path.to_str().unwrap().to_string();

                                BodyContent::File(state::FileContent::Content {
                                    path,
                                    info: FileInfo {
                                        name,
                                        size: m.size().to_string(),
                                        path: path_str,
                                    },
                                })
                            }
                            Err(_) => BodyContent::File(state::FileContent::None),
                        }
                    } else {
                        BodyContent::File(state::FileContent::None)
                    }
                }
            };

            reqs.push(RequestItem::new_v2(
                req.id,
                req.name,
                req.url,
                ParamsTable::from_model(req.headers),
                ParamsTable::from_model(req.params),
                req.method,
                body,
            ));
        }
        list.push(CollectionItem::new_v2(coll.id, coll.name).with_reqs(reqs));
    }

    Pane::new(
        (project.id, project.name),
        PaneState::new(
            CollectionsList::new().with_collections(list),
            Environments::from_model(project.environments),
            project.selected_env_context,
        ),
        draw_signal,
        task_sender,
    )
}
