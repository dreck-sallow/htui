use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    text::Span,
    Frame,
};

use crate::store::models::ProjectModel;

use super::{collections::CollectionsView, upsert_item::UpsertItemView};

pub struct PaneState {
    project: ProjectModel,
}

impl PaneState {
    pub fn new(project: ProjectModel) -> Self {
        Self { project }
    }
}

pub struct PaneView {
    state: PaneState,
    collections_view: CollectionsView,
    upsert_item_view: UpsertItemView,
}

impl PaneView {
    pub fn new(project: ProjectModel) -> Self {
        let mut collections_view = CollectionsView::new();

        for collection in project.collections() {
            collections_view.insert_collection(collection);
        }

        Self {
            state: PaneState::new(project),
            collections_view,
            upsert_item_view: UpsertItemView::new(),
        }
    }

    pub fn project(&self) -> &ProjectModel {
        &self.state.project
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let areas = Layout::horizontal([Constraint::Percentage(25), Constraint::Fill(1)])
            .spacing(1)
            .split(area);

        self.collections_view
            .render(self.project(), frame, areas[0]);
        frame.render_widget(Span::from("Hello world!"), areas[1]);
        self.upsert_item_view.draw(frame);
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        self.collections_view.handle_key(key);
        self.upsert_item_view.handle_key(key);
    }
}
