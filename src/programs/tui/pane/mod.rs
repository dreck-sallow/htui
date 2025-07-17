use collections::{state::Idx, CollectionsComponent};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use method_url_bar::MethodUrlBarComponent;
use placeholder::PlaceholderView;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    Frame,
};
use request_builder::RequestEditorComponent;
// use response_viewer::ResponseViewerView;
use state::{ElementFocus, PaneState};
use store::PaneStore;

use crate::store::models::ProjectModel;

use super::{
    common::component::{Drawable, Interactive, Painter, WithHistory},
    events::EventSender,
};

mod body_editor;
mod collections;
mod method_url_bar;
mod mutation_history;
mod mutations;
mod params_table;
mod placeholder;
mod request_builder;
mod response_viewer;
mod responses;
mod state;
mod store;
mod text_editor;

pub struct Pane {
    // mutations_history: MutationsHistoryV2,
    project_id: String,
    project_name: String,
    store: PaneStore,
    collections_component: CollectionsComponent,
    method_url_component: MethodUrlBarComponent,
    request_builder_component: RequestEditorComponent,
    // collections_view: CollectionsView,
    placeholder_view: PlaceholderView,
    focus: ElementFocus,
    // method_url_view: MethodUrlBarView,
    // request_editor_view: RequestEditorView,
    // response_viewer: ResponseViewerView,
    _sender: EventSender,
}

impl Pane {
    pub fn from_project(project: ProjectModel, sender: EventSender) -> Self {
        let idx = if project.collections.is_empty() {
            Idx::None
        } else {
            Idx::Parent(0)
        };

        Self {
            // mutations_history: MutationsHistoryV2::new(),
            project_id: project.id().to_string(),
            project_name: project.name().to_string(),
            store: PaneStore::new(),
            focus: ElementFocus::Collections,
            collections_component: CollectionsComponent::new(project.collections),
            method_url_component: MethodUrlBarComponent::new(),
            request_builder_component: RequestEditorComponent::new(),
            // collections_view: CollectionsView::new(idx),
            // method_url_view: MethodUrlBarView::new(),
            // request_editor_view: RequestEditorView::new(),
            // response_viewer: ResponseViewerView::new(),
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
        self.placeholder_view.set_render_area(placeholder_area);
        // self.response_viewer.set_render_area(content_areas[2]);
    }

    pub fn draw(&self, frame: &mut Frame) {
        let mut painter = Painter::new();
        self.collections_component.draw(&mut painter, self.focus);
        // self.collections_view.draw(&mut painter, &self.store);

        if self.collections_component.current_request().is_some() {
            self.method_url_component.draw(&mut painter, self.focus);
            self.request_builder_component
                .draw(&mut painter, self.focus);
            // self.response_viewer.draw(&mut painter, &self.state);
        } else {
            self.placeholder_view.draw(frame);
        }

        painter.draw(frame);
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if KeyCode::Char('u') == key.code && key.modifiers == KeyModifiers::ALT {
            match self.focus {
                state::ElementFocus::Collections => {
                    self.collections_component.undo();
                }
                state::ElementFocus::MethodUrlBar => self.method_url_component.undo(),
                state::ElementFocus::RequestBuilder => {
                    self.request_builder_component.undo();
                }
                state::ElementFocus::ResponseViewer => {}
            }
        } else if KeyCode::Char('y') == key.code && key.modifiers == KeyModifiers::ALT {
            match self.focus {
                state::ElementFocus::Collections => {
                    self.collections_component.redo();
                }
                state::ElementFocus::MethodUrlBar => {
                    self.method_url_component.redo();
                }
                state::ElementFocus::RequestBuilder => {
                    self.request_builder_component.redo();
                }
                state::ElementFocus::ResponseViewer => {}
            }
        } else {
            match self.focus {
                state::ElementFocus::Collections => {
                    if let Some(effect) = self.collections_component.on_key(key) {
                        match effect {
                            collections::CollectionEffect::NextFocus => {
                                self.focus = self.focus.next();
                            }
                            collections::CollectionEffect::PreviousFocus => {
                                self.focus = self.focus.previous();
                            }
                            collections::CollectionEffect::ChangeCurrentRequest => {
                                if let Some(req) = self.collections_component.current_request() {
                                    self.method_url_component.set_data(req.method(), req.url());
                                }
                            }
                        }
                    }
                }
                state::ElementFocus::MethodUrlBar => {
                    if let Some(effect) = self.method_url_component.on_key(key) {
                        match effect {
                            method_url_bar::MethodUrlEffect::NextFocus => {
                                self.focus = self.focus.next()
                            }
                            method_url_bar::MethodUrlEffect::PreviousFocus => {
                                self.focus = self.focus.previous()
                            }
                        }
                    }
                }
                state::ElementFocus::RequestBuilder => {
                    if let Some(effect) = self.request_builder_component.on_key(key) {
                        match effect {
                            request_builder::RequestEditorEffect::NextFocus => {
                                self.focus = self.focus.next()
                            }
                            request_builder::RequestEditorEffect::PreviousFocus => {
                                self.focus = self.focus.previous()
                            }
                        }
                    }
                }
                state::ElementFocus::ResponseViewer => {
                    todo!()
                }
            }
        }
    }
}
