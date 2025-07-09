use crossterm::event::KeyEvent;
use pane::PaneView;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, Borders, Tabs},
    Frame,
};
use tokio::sync::mpsc;

use crate::{programs::tui::events::Event, store::models::ProjectModel};

mod body_editor;
mod collections;
mod context;
mod editor;
mod focus;
mod method_url_bar;
mod pane;
mod pane_state;
mod placeholder;
mod request_builder;
mod response_viewer;

pub struct DashboardView {
    panes: Vec<PaneView>,
    selected: Option<usize>,
    tabs_area: Rect,
}

impl DashboardView {
    pub fn new() -> Self {
        Self {
            panes: Vec::new(),
            selected: None,
            tabs_area: Rect::default(),
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

    pub fn calculate_areas(&mut self, area: Rect) {
        let [tabs_area, pane_area] =
            Layout::vertical([Constraint::Length(2), Constraint::Fill(1)]).areas(area);
        self.tabs_area = tabs_area;

        // for pane in self.t
        for pane in &mut self.panes {
            pane.set_area(pane_area);
        }
    }

    pub fn add_pane_from_project(
        &mut self,
        project: ProjectModel,
        sender: mpsc::UnboundedSender<Event>,
    ) {
        self.add_pane(PaneView::new(project, sender));
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        // Draw the top header tabs
        let titles = self.panes.iter().map(|pane| pane.project_name());
        let tabs = Tabs::new(titles)
            .select(self.selected)
            .highlight_style(Style::default().blue().underlined())
            .block(Block::new().borders(Borders::BOTTOM))
            .divider(" - ");
        frame.render_widget(tabs, self.tabs_area);

        // Draw the current selected pane
        if let Some(pane) = self.current_pane_mut() {
            pane.draw(frame);
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if let Some(pane) = self.current_pane_mut() {
            pane.on_key(key);
        }
    }
}
