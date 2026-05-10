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
use state_v2::{
    collections::{
        BodyContent, BodyForm, CollectionItem, CollectionsList, FileContent, FileInfo, ListIdx,
        RequestItem,
    },
    param_item,
    responses::{self, ResponseStatus},
    table::{self, TableState},
    PaneState, SectionFocus,
};
// use state::{
//     BodyContent, BodyForm, CollectionItem, CollectionsList, Environments, FileInfo, PaneState,
//     ParamsTable, RequestItem,
// };

use crate::store::models::{ProjectModel, TimeId};

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
// mod state;
mod state_v2;

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
    // pub fn from_project(
    //     project: ProjectModel,
    //     draw_signal: DrawSignal,
    //     task_sender: TaskSender,
    // ) -> Self {
    //     Self::new(
    //         (project.id, project.name),
    //         PaneState::from_parts(
    //             project.collections,
    //             project.environments,
    //             project.selected_env_context,
    //         ),
    //         draw_signal,
    //         task_sender,
    //     )
    // }

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
                    .send_req_v2(self.project_id.to_string(), req, self._task_sender.clone())
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

    pub fn handle_response(&mut self, req_id: TimeId, req_res: ReqResponse) {
        let list = &mut self.state.responses.list;

        match req_res {
            ReqResponse::Err(txt) => {
                list.insert(req_id, ResponseStatus::Error(txt));
            }
            ReqResponse::Sucess(res) => {
                let mut headers = table::TableState::new();

                for (key, value) in res.headers {
                    headers.add_row(responses::ReadonlyHeader { key, value });
                }

                let response = responses::Response {
                    status: res.status,
                    status_text: res.status_text,
                    version: res.version,
                    duration: res.duration,
                    size_bytes: 10,
                    content_type: res.content_type,
                    headers,
                    body: match res.body {
                        super::app_event::Body::Text(s) => responses::ResponseBody::Text(s),
                        super::app_event::Body::Binary(items) => {
                            responses::ResponseBody::Binary(items)
                        }
                        super::app_event::Body::Empty => responses::ResponseBody::Empty,
                    },
                    cookies: TableState::new(),
                };
                list.insert(req_id, responses::ResponseStatus::Success(response));
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
                    let mut list = Vec::new();
                    for param in m {
                        list.push(param_item::ParamItem {
                            enable: param.enable,
                            key: param.key,
                            value: param.value,
                        });
                    }

                    BodyContent::FormUrlEncoded(TableState::from(list))
                }
                crate::store::models::RequestBody::FormData(m) => {
                    let mut list = Vec::new();
                    for param in m {
                        list.push((
                            param.is_file,
                            param_item::ParamItem {
                                enable: param.enable,
                                key: param.key,
                                value: param.value,
                            },
                        ));
                    }

                    BodyContent::FormData(BodyForm::from(list))
                }
                crate::store::models::RequestBody::File(path) => {
                    if path.is_file() {
                        match tokio::fs::metadata(&path).await {
                            Ok(m) => {
                                let name = path.file_name().unwrap().to_str().unwrap().to_string();
                                let path_str = path.to_str().unwrap().to_string();

                                BodyContent::File(FileContent::Content {
                                    path,
                                    info: FileInfo {
                                        name,
                                        size: m.size().to_string(),
                                        path: path_str,
                                    },
                                })
                            }
                            Err(_) => BodyContent::File(FileContent::None),
                        }
                    } else {
                        BodyContent::File(FileContent::None)
                    }
                }
            };

            let headers = {
                let mut table = TableState::new();
                for header in req.headers {
                    table.add_row(param_item::ParamItem {
                        enable: header.enable,
                        key: header.key,
                        value: header.value,
                    });
                }
                table
            };

            let params = {
                let mut table = TableState::new();
                for param in req.params {
                    table.add_row(param_item::ParamItem {
                        enable: param.enable,
                        key: param.key,
                        value: param.value,
                    });
                }
                table
            };

            reqs.push(RequestItem::new_v2(
                req.id, req.name, req.url, headers, params, req.method, body,
            ));
        }
        list.push(CollectionItem::new_v2(coll.id, coll.name).with_reqs(reqs));
    }

    Pane::new(
        (project.id, project.name),
        PaneState::new(CollectionsList::new().with_collections(list)),
        draw_signal,
        task_sender,
    )
}
