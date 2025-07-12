use collections::{state::Idx, CollectionsView};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use method_url_bar::MethodUrlBarView;
use placeholder::PlaceholderView;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    Frame,
};
use request_builder::RequestEditorView;
use response_viewer::ResponseViewerView;
use state::PaneState;
use store::{mutator::MutationsHistoryV2, PaneStore};

use crate::store::models::ProjectModel;

use super::{element_view::Painter, events::EventSender};

mod body_editor;
mod collections;
mod method_url_bar;
mod mutation_history;
mod mutations;
mod placeholder;
mod request_builder;
mod response_viewer;
mod responses;
mod state;
mod store;
mod text_editor;

pub struct Pane {
    mutations_history: MutationsHistoryV2,
    project_id: String,
    project_name: String,
    // state: PaneState,
    store: PaneStore,
    collections_view: CollectionsView,
    placeholder_view: PlaceholderView,
    method_url_view: MethodUrlBarView,
    request_editor_view: RequestEditorView,
    response_viewer: ResponseViewerView,
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
            mutations_history: MutationsHistoryV2::new(),
            project_id: project.id().to_string(),
            project_name: project.name().to_string(),
            // state: PaneState::from_collections(project.collections),
            store: PaneStore::from_collections(project.collections),
            collections_view: CollectionsView::new(idx),
            method_url_view: MethodUrlBarView::new(),
            request_editor_view: RequestEditorView::new(),
            response_viewer: ResponseViewerView::new(),
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
        self.collections_view.set_render_area(collections_area);
        self.placeholder_view.set_render_area(placeholder_area);
        self.method_url_view.set_render_area(content_areas[0]);
        self.request_editor_view.set_render_area(content_areas[1]);
        self.response_viewer.set_render_area(content_areas[2]);
    }

    pub fn draw(&self, frame: &mut Frame) {
        let mut painter = Painter::new();
        self.collections_view.draw(&mut painter, &self.store);

        if self.store.current_request().is_some() {
            // self.method_url_view.draw(&mut painter, &self.state);
            // self.request_editor_view.draw(&mut painter, &self.state);
            // self.response_viewer.draw(&mut painter, &self.state);
        } else {
            self.placeholder_view.draw(frame);
        }

        painter.draw(frame);
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if KeyCode::Char('u') == key.code && key.modifiers == KeyModifiers::ALT {
            self.mutations_history.go_back(&mut self.store);
        } else if KeyCode::Char('y') == key.code && key.modifiers == KeyModifiers::ALT {
            self.mutations_history.go_forward(&mut self.store);
        } else {
            let actions = match self.store.focus() {
                state::ElementFocus::Collections => {
                    self.collections_view.handle_key(key, &self.store)
                }
                state::ElementFocus::MethodUrlBar => {
                    todo!()
                    // self.method_url_view
                    //     .handle_key(key, &mut mutation_collector, &self.state)
                }
                state::ElementFocus::RequestBuilder => {
                    todo!()
                    // self.request_editor_view
                    //     .handle_key(key, &mut mutation_collector, &self.state)
                }
                state::ElementFocus::ResponseViewer => {
                    todo!()
                }
            };

            self.mutations_history
                .apply_from_list(actions, &mut self.store);
        }
    }
}
