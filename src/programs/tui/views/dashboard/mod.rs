use crossterm::event::KeyEvent;
use pane::PaneView;
use ratatui::{
    layout::{Constraint, Layout},
    style::{Style, Stylize},
    widgets::{Block, Borders, Tabs},
    Frame,
};

use crate::store::models::ProjectModel;

mod collections;
mod pane;
mod upsert_item;

pub struct DashboardState {
    panes: Vec<PaneView>,
    selected: Option<usize>,
}

impl DashboardState {
    pub fn new() -> Self {
        Self {
            panes: Vec::new(),
            selected: None,
        }
    }

    pub fn add_pane(&mut self, pane: PaneView) {
        self.panes.push(pane);
        self.selected = self.selected.is_none().then_some(0);
    }

    fn current_pane_mut(&mut self) -> Option<&mut PaneView> {
        if let Some(i) = self.selected {
            return self.panes.get_mut(i);
        }
        None
    }
}

pub struct DashboardView(DashboardState);

impl DashboardView {
    pub fn new() -> Self {
        Self(DashboardState::new())
    }

    pub fn add_pane_from_project(&mut self, project: ProjectModel) {
        self.0.add_pane(PaneView::new(project));
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let areas =
            Layout::vertical([Constraint::Length(2), Constraint::Fill(1)]).split(frame.area());

        // Draw the top header tabs
        let titles = self.0.panes.iter().map(|pane| pane.project().name());
        let tabs = Tabs::new(titles)
            .select(self.0.selected)
            .highlight_style(Style::default().blue().underlined())
            .block(Block::new().borders(Borders::BOTTOM))
            .divider(" - ");
        frame.render_widget(tabs, areas[0]);

        // Draw the current selected pane
        if let Some(pane) = self.0.current_pane_mut() {
            pane.draw(frame, areas[1]);
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if let Some(pane) = self.0.current_pane_mut() {
            pane.handle_key_event(key);
        }
    }
}
