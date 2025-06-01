use ratatui::{
    layout::{Constraint, Layout, Rect},
    text::Span,
    Frame,
};

use crate::store::models::ProjectModel;

use super::collections::CollectionsView;

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
    }
}
