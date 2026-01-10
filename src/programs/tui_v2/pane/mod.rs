use ratatui::{
    layout::{Constraint, Layout, Rect},
    Frame,
};

use crate::store::models::ProjectModel;

pub struct Pane {
    project_id: String,
    project_name: String,
}

impl Pane {
    pub fn from_project(project: ProjectModel) -> Self {
        Self {
            project_id: project.id,
            project_name: project.name,
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
    }
}
