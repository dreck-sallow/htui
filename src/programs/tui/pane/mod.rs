use collections::CollectionsView;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use mutation_history::{MutationCollector, MutationsHistory};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    Frame,
};
use state::PaneState;

use crate::store::models::ProjectModel;

use super::{element_view::Painter, events::EventSender};

mod collections;
mod mutation_history;
mod mutations;
mod responses;
mod state;

pub struct Pane {
    mutations_history: MutationsHistory,
    project_id: String,
    project_name: String,
    state: PaneState,
    collections_view: CollectionsView,
    _sender: EventSender,
}

impl Pane {
    pub fn from_project(project: ProjectModel, sender: EventSender) -> Self {
        Self {
            mutations_history: MutationsHistory::new(),
            project_id: project.id().to_string(),
            project_name: project.name().to_string(),
            state: PaneState::from_collections(project.collections),
            collections_view: CollectionsView::new(),
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
    }

    pub fn draw(&self, frame: &mut Frame) {
        let mut painter = Painter::new();
        self.collections_view.draw(&mut painter, &self.state);
        painter.draw(frame);
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if KeyCode::Char('u') == key.code && key.modifiers == KeyModifiers::ALT {
            self.mutations_history
                .go_back(&mut self.state, &self._sender);
        } else {
            let mut mutation_collector = MutationCollector::new();

            match self.state.focus() {
                state::ElementFocus::Collections => {
                    self.collections_view
                        .handle_key(key, &mut mutation_collector, &self.state)
                }
                state::ElementFocus::MethodUrlBar => todo!(),
                state::ElementFocus::RequestBuilder => todo!(),
                state::ElementFocus::ResponseViewer => todo!(),
            }

            self.mutations_history.apply_from_collector(
                mutation_collector,
                &mut self.state,
                &self._sender,
            );
        }
    }
}
