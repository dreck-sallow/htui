use collections::CollectionsSidebar;
use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    Frame,
};
use state::PaneState;

use crate::store::models::ProjectModel;
mod collections;
mod state;

pub struct Pane {
    project_id: String,
    project_name: String,
    state: PaneState,
    collection_sidebar: CollectionsSidebar,
}

impl Pane {
    pub fn from_project(project: ProjectModel) -> Self {
        Self {
            project_id: project.id,
            project_name: project.name,
            state: PaneState::from_parts(
                project.collections,
                project.environments,
                project.selected_env_context,
            ),
            collection_sidebar: CollectionsSidebar::new(),
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

        self.collection_sidebar
            .draw(sidebar_area, frame, &self.state);
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match self.state.focus {
            state::SectionFocus::Collections => {
                self.collection_sidebar.handle_key(key, &mut self.state);
            }
            state::SectionFocus::UpsertItem => {}
        }

        true
    }
}
