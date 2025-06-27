use crossterm::event::KeyEvent;
use pane::PaneView;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, Borders, Tabs},
    Frame,
};

use crate::{programs::tui::element_view::ElementView, store::models::ProjectModel};

mod action;
mod collections;
mod editor;
mod focus;
mod global_pane_state;
mod method_selector;
mod pane;
mod pane_state;
mod request_builder;
mod response_viewer;
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

pub struct DashboardView {
    state: DashboardState,
    tabs_area: Rect,
}

impl DashboardView {
    pub fn new() -> Self {
        Self {
            state: DashboardState::new(),
            tabs_area: Rect::default(),
        }
    }

    pub fn calculate_areas(&mut self, area: Rect) {
        let [tabs_area, pane_area] =
            Layout::vertical([Constraint::Length(2), Constraint::Fill(1)]).areas(area);
        self.tabs_area = tabs_area;

        // for pane in self.t
        for pane in &mut self.state.panes {
            pane.on_resize(pane_area, &mut ()); // QUEST: call om_resize?
        }
    }

    pub fn add_pane_from_project(&mut self, project: ProjectModel) {
        self.state.add_pane(PaneView::new(project));
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        // let areas =
        //     Layout::vertical([Constraint::Length(2), Constraint::Fill(1)]).split(frame.area());

        // Draw the top header tabs
        let titles = self.state.panes.iter().map(|pane| pane.project_name());
        let tabs = Tabs::new(titles)
            .select(self.state.selected)
            .highlight_style(Style::default().blue().underlined())
            .block(Block::new().borders(Borders::BOTTOM))
            .divider(" - ");
        frame.render_widget(tabs, self.tabs_area);

        // Draw the current selected pane
        if let Some(pane) = self.state.current_pane_mut() {
            pane.draw(frame, &mut ());
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if let Some(pane) = self.state.current_pane_mut() {
            pane.on_key(key, &mut ());
        }
    }
}
