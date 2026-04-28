use crossterm::event::{KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Style, Stylize},
    widgets::{Block, Borders, Tabs},
    Frame,
};

use crate::store::models::ProjectModel;

use super::{
    app_event::{ReqResponse, TaskSender},
    events::DrawSignal,
    pane::{new_pane, Pane},
};

#[derive(Default)]
pub struct TuiApp {
    panes: Vec<Pane>,
    selected: Option<usize>,
}

impl TuiApp {
    pub fn add_project(
        &mut self,
        project: ProjectModel,
        draw_signal: DrawSignal,
        task_sender: TaskSender,
    ) {
        let pane = Pane::from_project(project, draw_signal, task_sender);

        self.panes.push(pane);

        if self.selected.is_none() {
            self.selected = Some(0);
        }
    }

    pub async fn add_project_v2(
        &mut self,
        project: ProjectModel,
        draw_signal: DrawSignal,
        task_sender: TaskSender,
    ) {
        let pane = new_pane(project, draw_signal, task_sender).await;

        self.panes.push(pane);

        if self.selected.is_none() {
            self.selected = Some(0);
        }
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

    pub async fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind == KeyEventKind::Press {
            match key.code {
                crossterm::event::KeyCode::Char(ch) => {
                    if ch == 'c' && key.modifiers == KeyModifiers::CONTROL {
                        return false;
                    }
                }
                _ => {}
            }

            // Draw the current selected pane
            if let Some(pane) = self.selected.and_then(|i| self.panes.get_mut(i)) {
                pane.handle_key(key).await;
            }
        }

        true
    }

    pub fn handle_response(&mut self, res: ReqResponse) {
        // Draw the current selected pane
        if let Some(pane) = self.selected.and_then(|i| self.panes.get_mut(i)) {
            pane.handle_response(res);
        }
    }

    pub fn handle_quit(&self) {}
}
