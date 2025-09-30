use std::io::Stdout;

use action::PaneAction;
use arboard::Clipboard;
use collections::CollectionsComponent;
use crossterm::event::KeyEvent;
use method_url_bar::MethodUrlBarComponent;
use placeholder::PlaceholderView;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    prelude::CrosstermBackend,
    Frame, Terminal,
};
use request_builder::RequestEditorComponent;
use request_response::ResponseViewerComponent;
use tokio::sync::mpsc;

use crate::app_project::{
    models::ProjectModel,
    store::{LocalStore, Store},
};

use super::{
    common::{component::WithHistory, Interactive, UiComposedElement},
    config::{keybinding, Config},
    event_handler::{AppMessage, Events},
};

mod action;
mod collections;
mod method_url_bar;
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
    // config: Rc<Config>,
    sender: mpsc::Sender<AppMessage>,
}

impl Pane {
    pub fn from_project(
        project: ProjectModel,
        config: &Config,
        sender: mpsc::Sender<AppMessage>,
    ) -> Self {
        Self {
            project_id: project.id().to_string(),
            project_name: project.name().to_string(),
            focus: ElementFocus::Collections,
            collections_component: CollectionsComponent::new(project.collections),
            method_url_component: MethodUrlBarComponent::new(config),
            request_builder_component: RequestEditorComponent::new(config),
            response_viewer_component: ResponseViewerComponent::new(),
            placeholder_view: PlaceholderView::new(),
            // config,
            sender: sender,
        }
    }

    pub fn project_name(&self) -> &str {
        &self.project_name
    }
    pub fn set_project_name(&mut self, name: String) {
        self.project_name = name;
    }

    pub fn id(&self) -> String {
        self.project_id.clone()
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
                if self.focus == ElementFocus::Collections {
                    if self.collections_component.current_request().is_some() {
                        self.focus = self.focus.next();
                    }
                } else if self.focus == ElementFocus::RequestBuilder {
                    // TODO: hadle next focus when no request sent
                    self.focus = self.focus.next();
                } else {
                    self.focus = self.focus.next();
                }
            }
            PaneAction::PreviousFocus => {
                self.focus = self.focus.previous();
            }
            PaneAction::ChangeRequest => {
                if let Some(req) = self.collections_component.current_request() {
                    self.method_url_component.set_data(req.method(), req.url());
                    self.request_builder_component.set_state(
                        req.params.clone(),
                        req.headers().to_vec(),
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
                let (method, url) = self.method_url_component.get_data();
                self.collections_component
                    .set_data_from_method_url(method, url);

                let (params, headers, body) = self.request_builder_component.get_data();
                self.collections_component
                    .set_data_from_request_editor(params, headers, body);

                if let Some(req) = self.collections_component.current_request() {
                    self.response_viewer_component.execute_req(
                        self.collections_component.current_request_key().unwrap(),
                        req,
                        self.sender.clone(),
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

                let store = LocalStore::new();
                let _ = store.save_project(model);
            }
            PaneAction::Noop => {}
        }
    }

    fn handle_pane_action(&mut self, config: &Config, key: KeyEvent) -> bool {
        if let Some(key_action) = config.keymap.match_global_action(key) {
            match key_action {
                keybinding::GlobalKeyAction::Undo => {
                    if let Some(component) = self.get_with_history_component() {
                        component.undo();
                    }
                }
                keybinding::GlobalKeyAction::Redo => {
                    if let Some(component) = self.get_with_history_component() {
                        component.redo();
                    }
                }
                keybinding::GlobalKeyAction::SendRequest => {
                    self.handle_action(PaneAction::ExecuteRequest);
                }
                keybinding::GlobalKeyAction::SaveProject => {
                    self.handle_action(PaneAction::SaveLocal);
                }
                _ => return false,
            }

            return true;
        }

        false
    }
}

impl<'params> UiComposedElement<'params> for Pane {
    type Params = &'params Config;

    fn set_area(&mut self, area: Rect, viewport_area: Rect) {
        let (collections_area, placeholder_area, content_areas) = {
            let [collections_area, content_area] =
                Layout::horizontal([Constraint::Percentage(23), Constraint::Fill(1)])
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
        self.collections_component
            .set_area(collections_area, viewport_area);
        self.method_url_component
            .set_area(content_areas[0], viewport_area);
        self.request_builder_component
            .set_area(content_areas[1], viewport_area);
        self.response_viewer_component
            .set_area(content_areas[2], viewport_area);
        self.placeholder_view.set_render_area(placeholder_area);
    }

    fn draw(&self, config: Self::Params, frame: &mut Frame) {
        self.collections_component.draw((self.focus, config), frame);

        if self.collections_component.current_request().is_some() {
            self.method_url_component.draw((self.focus, config), frame);
            self.request_builder_component
                .draw((self.focus, config), frame);
            self.response_viewer_component
                .draw((self.focus, config), frame);
        } else {
            self.placeholder_view.draw(frame);
        }

        self.collections_component
            .draw_overlay((self.focus, config), frame);
        self.method_url_component
            .draw_overlay((self.focus, config), frame);
        self.request_builder_component
            .draw_overlay((self.focus, config), frame);
        self.response_viewer_component
            .draw_overlay((self.focus, config), frame);
    }
}

impl<'params> Interactive<'params> for Pane {
    type Effect = ();
    type Params = (
        &'params Config,
        &'params mut Events<AppMessage>,
        &'params mut Terminal<CrosstermBackend<Stdout>>,
        &'params mut Clipboard,
    );

    fn handle_key(
        &mut self,
        (config, events, terminal, clipboard): Self::Params,
        key: KeyEvent,
    ) -> Self::Effect {
        // TODO: handle the undo of creation
        // CONTEXT: when I create a request and exeute it, and after, make an undo
        // the system delete the request, but the background task is still running

        if !self.handle_pane_action(config, key) {
            match self.focus {
                ElementFocus::Collections => {
                    let effect = self.collections_component.handle_key(config, key);
                    self.handle_action(effect);
                }
                ElementFocus::MethodUrlBar => {
                    let effect = self.method_url_component.handle_key(config, key);
                    self.handle_action(effect);
                    self.handle_action(PaneAction::SetUrlAndMethod);
                }
                ElementFocus::RequestBuilder => {
                    let effect = self
                        .request_builder_component
                        .handle_key((config, events, terminal, clipboard), key);
                    self.handle_action(effect);
                    self.handle_action(PaneAction::SetHeadersAndBody);
                }
                ElementFocus::ResponseViewer => {
                    let effect = self
                        .response_viewer_component
                        .handle_key((config, events, terminal, clipboard), key);
                    self.handle_action(effect);
                }
            }
        }
    }

    fn is_input_focus(&self) -> bool {
        false
    }
}
