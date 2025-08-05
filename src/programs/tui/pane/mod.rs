use crate::{
    paths::Paths,
    store::{
        models::{KeyValueParam, ProjectModel},
        LocalStore, Store,
    },
};
use action::PaneAction;
use collections::CollectionsComponent;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use method_url_bar::MethodUrlBarComponent;
use placeholder::PlaceholderView;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    Frame,
};
use request_builder::RequestEditorComponent;
use request_response::ResponseViewerComponent;

use super::{
    common::component::{Drawable, Interactive, Painter, WithHistory},
    events::EventSender,
};

mod action;
mod body_editor;
mod collections;
mod method_url_bar;
mod params_table;
mod placeholder;
mod request_builder;
mod request_response;
mod text_editor;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ElementFocus {
    Collections,
    MethodUrlBar,
    RequestBuilder,
    ResponseViewer,
}

impl ElementFocus {
    pub fn next(&self) -> Self {
        match self {
            ElementFocus::Collections => ElementFocus::MethodUrlBar,
            ElementFocus::MethodUrlBar => ElementFocus::RequestBuilder,
            ElementFocus::RequestBuilder => ElementFocus::ResponseViewer,
            ElementFocus::ResponseViewer => ElementFocus::Collections,
        }
    }

    pub fn previous(&self) -> Self {
        match self {
            ElementFocus::Collections => ElementFocus::ResponseViewer,
            ElementFocus::MethodUrlBar => ElementFocus::Collections,
            ElementFocus::RequestBuilder => ElementFocus::MethodUrlBar,
            ElementFocus::ResponseViewer => ElementFocus::RequestBuilder,
        }
    }
}

pub struct Pane {
    project_id: String,
    project_name: String,
    collections_component: CollectionsComponent,
    method_url_component: MethodUrlBarComponent,
    request_builder_component: RequestEditorComponent,
    response_viewer_component: ResponseViewerComponent,
    placeholder_view: PlaceholderView,
    focus: ElementFocus,
    _sender: EventSender,
}

impl Pane {
    pub fn from_project(project: ProjectModel, sender: EventSender) -> Self {
        Self {
            project_id: project.id().to_string(),
            project_name: project.name().to_string(),
            focus: ElementFocus::Collections,
            collections_component: CollectionsComponent::new(project.collections),
            method_url_component: MethodUrlBarComponent::new(),
            request_builder_component: RequestEditorComponent::new(),
            response_viewer_component: ResponseViewerComponent::new(),
            placeholder_view: PlaceholderView::new(),
            _sender: sender,
        }
    }

    pub fn project_name(&self) -> &str {
        &self.project_name
    }

    pub fn set_render_area(&mut self, area: Rect) {
        let (collections_area, placeholder_area, content_areas) = {
            let [collections_area, content_area] =
                Layout::horizontal([Constraint::Percentage(25), Constraint::Fill(1)])
                    .spacing(1)
                    .areas(area);

            let right_areas = Layout::vertical([
                Constraint::Length(3),
                Constraint::Fill(1),
                Constraint::Fill(1),
            ])
            .split(content_area);

            (collections_area, content_area, right_areas)
        };
        self.collections_component.set_area(collections_area);
        self.method_url_component.set_area(content_areas[0]);
        self.request_builder_component.set_area(content_areas[1]);
        self.response_viewer_component.set_area(content_areas[2]);
        self.placeholder_view.set_render_area(placeholder_area);
    }

    pub fn draw(&self, frame: &mut Frame) {
        let mut painter = Painter::new();
        self.collections_component.draw(&mut painter, self.focus);

        if self.collections_component.current_request().is_some() {
            self.method_url_component.draw(&mut painter, self.focus);
            self.request_builder_component
                .draw(&mut painter, self.focus);
            self.response_viewer_component
                .draw(&mut painter, self.focus);
        } else {
            self.placeholder_view.draw(frame);
        }

        painter.draw(frame);
    }

    pub fn get_with_history_component(&mut self) -> Option<Box<&mut dyn WithHistory>> {
        match self.focus {
            ElementFocus::Collections => Some(Box::new(&mut self.collections_component)),
            ElementFocus::MethodUrlBar => Some(Box::new(&mut self.method_url_component)),
            ElementFocus::RequestBuilder => Some(Box::new(&mut self.request_builder_component)),
            ElementFocus::ResponseViewer => None,
        }
    }

    pub fn handle_action(&mut self, action: PaneAction) {
        match action {
            PaneAction::NextFocus => {
                self.focus = self.focus.next();
            }
            PaneAction::PreviousFocus => {
                self.focus = self.focus.previous();
            }
            PaneAction::ChangeRequest => {
                if let Some(req) = self.collections_component.current_request() {
                    self.method_url_component.set_data(req.method(), req.url());
                    let headers = req
                        .headers()
                        .iter()
                        .map(|(k, v)| KeyValueParam::new(k.to_string(), v.to_string()))
                        .collect();

                    self.request_builder_component.set_state(
                        req.params.clone(),
                        headers,
                        req.body().clone(),
                    );

                    self.response_viewer_component
                        .change_req(self.collections_component.current_request_key().unwrap());
                }
            }
            PaneAction::DeleteRequest => {
                // TODO: cancel the background http request
            }
            PaneAction::ExecuteRequest => {
                if let Some(req) = self.collections_component.current_request() {
                    self.response_viewer_component.execute_req(
                        self.collections_component.current_request_key().unwrap(),
                        req,
                        self._sender.clone(),
                    );
                }
            }
            PaneAction::SetUrlAndMethod => {
                let (method, url) = self.method_url_component.get_data();
                self.collections_component
                    .set_data_from_method_url(method, url);
            }
            PaneAction::SetHeadersAndBody => {
                let (params, headers, body) = self.request_builder_component.get_data();
                self.collections_component
                    .set_data_from_request_editor(params, headers, body);
            }
            PaneAction::SaveLocal => {
                let model = ProjectModel::from_parts(
                    self.project_id.clone(),
                    self.project_name.clone(),
                    self.collections_component.as_collections(),
                );

                // QUESTION: We need the store async?
                let _ = tokio::task::spawn_blocking(move || {
                    let rt = tokio::runtime::Handle::current();
                    let store = LocalStore::new(Paths::new("store"));
                    rt.block_on(async move {
                        let _ = store.save_project(model).await;
                    })
                });
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        // TODO: handle the undo of creation
        // Context: when I create a request and exeute it, and after, make an undo
        // the system delete the request, but the background task is still running

        if KeyCode::Char('u') == key.code && key.modifiers == KeyModifiers::ALT {
            if let Some(component) = self.get_with_history_component() {
                component.undo();
            }
        } else if KeyCode::Char('y') == key.code && key.modifiers == KeyModifiers::ALT {
            if let Some(component) = self.get_with_history_component() {
                component.redo();
            }
        } else if KeyCode::Char('x') == key.code && key.modifiers == KeyModifiers::ALT {
            self.handle_action(PaneAction::ExecuteRequest);
        } else if KeyCode::Char('s') == key.code && key.modifiers == KeyModifiers::ALT {
            self.handle_action(PaneAction::SaveLocal);
        } else {
            match self.focus {
                ElementFocus::Collections => {
                    if let Some(effect) = self.collections_component.on_key(key) {
                        self.handle_action(effect);
                    }
                }
                ElementFocus::MethodUrlBar => {
                    if let Some(effect) = self.method_url_component.on_key(key) {
                        self.handle_action(effect);
                        self.handle_action(PaneAction::SetUrlAndMethod);
                    }
                }
                ElementFocus::RequestBuilder => {
                    if let Some(effect) = self.request_builder_component.on_key(key) {
                        self.handle_action(effect);
                        self.handle_action(PaneAction::SetHeadersAndBody);
                    }
                }
                ElementFocus::ResponseViewer => {
                    if let Some(effect) = self.response_viewer_component.on_key(key) {
                        self.handle_action(effect);
                    }
                }
            }
        }
    }
}
