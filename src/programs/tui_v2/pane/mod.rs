use actions::EffectsCollector;
use collections::CollectionsSidebar;
use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    Frame,
};
use request_bar::RequestBar;
use request_builder::RequestBuilder;
use state::PaneState;

use crate::store::models::ProjectModel;

use super::events::DrawSignal;
mod actions;
mod collections;
mod common;
mod request_bar;
mod request_builder;
mod state;

pub struct Pane {
    project_id: String,
    project_name: String,
    state: PaneState,
    collection_sidebar: CollectionsSidebar,
    request_bar: RequestBar,
    request_builder: RequestBuilder,
}

impl Pane {
    pub fn from_project(project: ProjectModel, draw_signal: DrawSignal) -> Self {
        Self {
            project_id: project.id,
            project_name: project.name,
            state: PaneState::from_parts(
                project.collections,
                project.environments,
                project.selected_env_context,
            ),
            collection_sidebar: CollectionsSidebar::new(),
            request_bar: RequestBar::new(),
            request_builder: RequestBuilder::new(draw_signal),
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

        match self.state.collections.idx {
            state::ListIdx::None | state::ListIdx::Group(_) => {}
            state::ListIdx::Item(_, _) => {
                let [bar_area, builder_area] =
                    Layout::vertical([Constraint::Length(3), Constraint::Percentage(40)])
                        .areas(main_area);
                self.request_bar.draw(bar_area, frame, &self.state);
                self.request_builder.draw(builder_area, frame, &self.state);
            }
        }

        self.collection_sidebar
            .draw_overlay(area, frame, &self.state);
        self.request_bar.draw_overlay(area, frame);
        self.request_builder.draw_overlay(frame);
    }

    pub async fn handle_key(&mut self, key: KeyEvent) -> bool {
        let mut effects = EffectsCollector::new();

        match self.state.focus {
            state::SectionFocus::Collections => {
                self.collection_sidebar
                    .handle_key(key, &mut self.state, &mut effects);
            }
            state::SectionFocus::RequestBar => {
                self.request_bar.handle_key(key, &mut self.state);
            }
            state::SectionFocus::RequestBuilder => {
                self.request_builder.handle_key(key, &mut self.state).await;
            }
        }

        while let Some(effect) = effects.next() {
            match effect {
                actions::PaneActionEffect::ChangeIdx => {
                    self.request_bar.sync(&mut self.state);
                    self.request_builder.sync(&mut self.state);
                }
            }
        }

        true
    }
}
