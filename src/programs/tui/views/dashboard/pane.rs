use ratatui::{layout::Rect, text::Span, Frame};

use crate::store::models::ProjectModel;

pub struct PaneState {
    project: ProjectModel,
}

impl PaneState {
    pub fn new(project: ProjectModel) -> Self {
        Self { project }
    }
}

pub struct PaneView(PaneState);

impl PaneView {
    pub fn new(project: ProjectModel) -> Self {
        Self(PaneState::new(project))
    }

    pub fn project(&self) -> &ProjectModel {
        &self.0.project
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_widget(Span::from("Hello world!"), area);
    }
}
