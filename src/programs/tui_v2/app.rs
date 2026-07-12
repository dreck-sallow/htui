use crossterm::event::{KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Style, Stylize},
    widgets::{Block, Borders, Tabs},
    Frame,
};

use crate::store::models::ProjectModel;

use super::{
    ctx::InitialCtx,
    pane::{new_pane, Pane},
    task::TaskResult,
};

pub type TaskGroupKey = String;

pub struct TuiApp {
    panes: Vec<Pane>,
    selected: Option<usize>,
}

impl TuiApp {
    pub async fn new_from_project(project: ProjectModel, ctx: InitialCtx) -> Self {
        let pane = new_pane(project, ctx).await;

        Self {
            panes: vec![pane],
            selected: Some(0),
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

    pub fn handle_task(&mut self, task: TaskResult<TaskGroupKey>) {
        for pane in &mut self.panes {
            if pane.id() == &task.group_key {
                match task.result {
                    super::task::TaskResultType::HttpResponse { req_id, res } => {
                        pane.handle_response_v2(req_id, res);
                    }
                    super::task::TaskResultType::HttpExecError { req_id, error } => {
                        pane.handle_http_error(req_id, error);
                    }
                }
                break;
            }
        }
    }

    pub fn handle_quit(&self) {}
}
