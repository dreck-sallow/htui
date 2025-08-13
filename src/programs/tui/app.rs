use std::{collections::HashSet, rc::Rc};

use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, Borders, Tabs},
    Frame,
};

use crate::store::models::ProjectModel;

use super::{
    app_components::SearchProjects,
    common::component::{Drawable, Interactive, Painter},
    config::{keybinding, Config},
    events::EventSender,
    pane::Pane,
};

pub struct App {
    panes: Vec<Pane>,
    selected: Option<usize>,
    tabs_area: Rect,
    search_projects: SearchProjects,
    config: Rc<Config>,
    show_search_projects: bool,
}

impl App {
    pub fn new_from_project(
        project: ProjectModel,
        config: Rc<Config>,
        sender: EventSender,
    ) -> Self {
        let mut this = Self::new(Rc::clone(&config));
        this.add_project(project, sender);
        this
    }

    fn new(config: Rc<Config>) -> Self {
        Self {
            panes: Vec::new(),
            selected: None,
            tabs_area: Rect::default(),
            search_projects: SearchProjects::new(Rc::clone(&config)),
            show_search_projects: false,
            config,
        }
    }

    fn add_project(&mut self, project: ProjectModel, sender: EventSender) {
        self.panes
            .push(Pane::from_project(project, Rc::clone(&self.config), sender));
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

        if self.show_search_projects {
            let mut painter = Painter::new();
            self.search_projects.draw(&mut painter, ());
            painter.draw(frame);
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent, sender: EventSender) {
        if self.show_search_projects {
            if let Some(effect) = self.search_projects.on_key(key) {
                match effect {
                    super::app_components::SearchProjectsEffect::Submit(_project) => {
                        // TODO: replace the current ProjectModel from app_project::models::ProjectModel
                        self.add_project(ProjectModel::new(_project.id().into()), sender);
                    }
                    super::app_components::SearchProjectsEffect::Hidden => {
                        self.show_search_projects = false;
                    }
                }
            }
        } else {
            match self.config.keymap.match_app_action(key) {
                Some(key) => match key {
                    keybinding::AppKeyAction::SearchProject => {
                        let mut openeds = HashSet::new();

                        for pane in &self.panes {
                            openeds.insert(pane.id());
                        }
                        self.search_projects.search(openeds);
                        self.show_search_projects = true;
                    }
                    keybinding::AppKeyAction::DeleteProject => {}
                },
                None => {
                    if let Some(pane) = self.current_pane_mut() {
                        pane.handle_key(key);
                    }
                }
            }
        }
    }
}
