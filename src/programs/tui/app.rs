use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, Borders, Tabs},
    Frame,
};

use crate::store::models::ProjectModel;

use super::{events::EventSender, pane::Pane};

pub struct App {
    panes: Vec<Pane>,
    selected: Option<usize>,
    tabs_area: Rect,
}

impl App {
    pub fn new_from_project(project: ProjectModel, sender: EventSender) -> Self {
        let mut this = Self::new();
        this.add_project(project, sender);
        this
    }

    fn new() -> Self {
        Self {
            panes: Vec::new(),
            selected: None,
            tabs_area: Rect::default(),
        }
    }

    fn add_project(&mut self, project: ProjectModel, sender: EventSender) {
        self.panes.push(Pane::from_project(project, sender));
        if self.selected.is_none() {
            self.selected = Some(0);
        }
    }

    fn current_pane(&self) -> Option<&Pane> {
        self.selected.and_then(|i| self.panes.get(i))
    }

    fn current_pane_mut(&mut self) -> Option<&mut Pane> {
        self.selected.and_then(|i| self.panes.get_mut(i))
    }

    pub fn viewport_area(&mut self, viewport: Rect) {
        let [tabs_area, pane_area] =
            Layout::vertical([Constraint::Length(2), Constraint::Fill(1)]).areas(viewport);
        self.tabs_area = tabs_area;

        // for pane in self.t
        for pane in &mut self.panes {
            pane.set_render_area(pane_area);
        }
    }

    pub fn handle_draw(&self, frame: &mut Frame) {
        // Draw the top header tabs
        let titles = self.panes.iter().map(|pane| pane.project_name());
        let tabs = Tabs::new(titles)
            .select(self.selected)
            .highlight_style(Style::default().blue().underlined())
            .block(Block::new().borders(Borders::BOTTOM))
            .divider(" - ");

        frame.render_widget(tabs, self.tabs_area);

        // Draw the current selected pane
        if let Some(pane) = self.current_pane() {
            pane.draw(frame);
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if let Some(pane) = self.current_pane_mut() {
            pane.handle_key(key);
        }
    }
}
