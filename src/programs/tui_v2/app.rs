use crossterm::event::{KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Style, Stylize},
    widgets::{Block, Borders, Tabs},
    Frame,
};

use crate::store::models::ProjectModel;

use super::{events::DrawSignal, pane::Pane};

#[derive(Default)]
pub struct TuiApp {
    panes: Vec<Pane>,
    selected: Option<usize>,
}

impl TuiApp {
    pub fn add_project(mut self, project: ProjectModel) -> Self {
        let pane = Pane::from_project(project);

        self.panes.push(pane);

        if self.selected.is_none() {
            self.selected = Some(0);
        }

        self
    }
}

impl TuiApp {
    pub fn handle_draw(&self, frame: &mut Frame) {
        let [tabs_area, pane_body_area] =
            Layout::vertical([Constraint::Length(2), Constraint::Fill(1)]).areas(frame.area());

        // Draw the top header tabs
        let titles = self.panes.iter().map(|pane| pane.name());
        let tabs = Tabs::new(titles)
            .select(self.selected)
            .highlight_style(Style::default().blue().underlined())
            .block(Block::new().borders(Borders::BOTTOM))
            .divider(" - ");

        frame.render_widget(tabs, tabs_area);

        // Draw the current selected pane
        if let Some(pane) = self.selected.and_then(|i| self.panes.get(i)) {
            pane.draw(pane_body_area, frame);
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent, draw_signal: DrawSignal) -> bool {
        if key.kind == KeyEventKind::Press {
            match key.code {
                crossterm::event::KeyCode::Char(ch) => {
                    if ch == 'c' && key.modifiers == KeyModifiers::CONTROL {
                        return false;
                    }
                }
                _ => {}
            }
        }

        true
    }

    pub fn handle_quit(&self) {}
}
