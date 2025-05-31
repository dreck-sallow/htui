use std::{collections::HashSet, io::Stdout};

use arboard::Clipboard;
use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    prelude::CrosstermBackend,
    style::{Style, Stylize},
    widgets::{Block, Borders, Clear, Tabs},
    Frame, Terminal,
};
use tokio::sync::mpsc;

use crate::app_project::models::ProjectModel;

use super::{
    app_components::SearchProjects,
    common::{
        input::mode_input::ModeInput,
        list_utils::{next, prev},
        Interactive, UiComposedElement, UiElementV2,
    },
    config::{keybinding, Config},
    elements::utils::center_area,
    event_handler::{AppMessage, Events},
    pane::Pane,
};

pub struct App {
    panes: Vec<Pane>,
    selected: Option<usize>,
    tabs_area: Rect,
    search_projects: SearchProjects,
    project_name_input: ModeInput,
    config: Config,
    show_search_projects: bool,
    show_project_name_input: bool,
    pane_area: Rect,
    viewport: Rect,
}

impl App {
    pub fn new_from_project(
        project: ProjectModel,
        config: Config,
        sender: mpsc::Sender<AppMessage>,
    ) -> Self {
        let mut this = Self::new(config);
        this.add_project(project, sender);
        this
    }

    fn new(config: Config) -> Self {
        Self {
            panes: Vec::new(),
            selected: None,
            tabs_area: Rect::default(),
            search_projects: SearchProjects::new(),
            show_search_projects: false,
            pane_area: Rect::default(),
            viewport: Rect::default(),
            project_name_input: ModeInput::new(""),
            show_project_name_input: false,
            config,
        }
    }

    fn add_project(&mut self, project: ProjectModel, sender: mpsc::Sender<AppMessage>) {
        let mut pane = Pane::from_project(project, &self.config, sender);
        pane.set_area(self.pane_area, self.viewport);
        self.panes.push(pane);
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
        self.viewport = viewport;
        let [tabs_area, pane_area] =
            Layout::vertical([Constraint::Length(2), Constraint::Fill(1)]).areas(viewport);
        self.tabs_area = tabs_area;
        self.pane_area = pane_area;

        // for pane in self.t
        for pane in &mut self.panes {
            pane.set_area(pane_area, viewport);
        }
        self.project_name_input.set_visual_width(
            center_area(viewport, Constraint::Length(3), Constraint::Percentage(40))
                .width
                .saturating_sub(2),
        );
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
            pane.draw(&self.config, frame);
        }

        if self.show_search_projects {
            self.search_projects.draw(&self.config, frame);
        } else if self.show_project_name_input {
            let area = center_area(
                frame.area(),
                Constraint::Length(3),
                Constraint::Percentage(40),
            );
            let block = Block::bordered()
                .title(format!(" {} ", "Project Name"))
                .border_style(Style::default().blue());

            let inner_area = block.inner(area);
            frame.render_widget(Clear, area);
            frame.render_widget(block, area);
            self.project_name_input.draw(inner_area, frame);
        }
    }

    pub fn handle_key(
        &mut self,
        key: KeyEvent,
        events: &mut Events<AppMessage>,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
        clipboard: &mut Clipboard,
    ) {
        if self.show_search_projects {
            if let Some(effect) = self.search_projects.handle_key(&self.config, key) {
                match effect {
                    super::app_components::SearchProjectsEffect::Submit(project) => {
                        self.add_project(project, events.sender());
                        self.selected = next(self.selected, self.panes.len());
                        self.show_search_projects = false;
                    }
                    super::app_components::SearchProjectsEffect::Hidden => {
                        self.show_search_projects = false;
                    }
                }
            }
        } else if self.show_project_name_input {
            if let Some(key_action) = self.config.keymap.match_global_action(key) {
                match key_action {
                    keybinding::GlobalKeyAction::ClosePopup => {
                        self.project_name_input.clear();
                        self.show_project_name_input = false;
                    }
                    keybinding::GlobalKeyAction::SubmitPopup => {
                        let renamed = self.project_name_input.txt().to_owned();
                        self.current_pane_mut().unwrap().set_project_name(renamed);

                        self.project_name_input.clear();
                        self.show_project_name_input = false;
                    }
                    _ => {
                        self.project_name_input.handle_key(key);
                    }
                }
            } else {
                self.project_name_input.handle_key(key);
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
                    keybinding::AppKeyAction::CloseProject => {
                        // TODO: alert user when the data is not saved?
                        if let Some(idx) = self.selected {
                            // when close las project, finish the program?
                            if self.panes.len() > 1 {
                                // TODO: handle the stop pending requests!
                                self.panes.remove(idx);
                                self.selected = Some(if idx == 0 { 0 } else { idx - 1 });
                            }
                        }
                    }
                    keybinding::AppKeyAction::NextProject => {
                        self.selected = next(self.selected, self.panes.len());
                    }
                    keybinding::AppKeyAction::PreviousProject => {
                        self.selected = prev(self.selected);
                    }
                    keybinding::AppKeyAction::RenameProject => {
                        if let Some(name) = self.current_pane().map(|p| p.project_name().to_owned())
                        {
                            self.project_name_input.replace(&name);
                            // self.project_name_input.insert_str(&name);
                            self.show_project_name_input = true;
                        }
                    }
                },
                None => {
                    // FIXME: why I cannot use self.current_pane_mut()?
                    if let Some(pane) = self.selected.and_then(|i| self.panes.get_mut(i)) {
                        pane.handle_key((&self.config, events, terminal, clipboard), key);
                    }
                }
            }
        }
    }
}
