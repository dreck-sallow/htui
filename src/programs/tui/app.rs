use std::{cell::RefCell, collections::HashSet, io::Stdout, rc::Rc};

use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    prelude::CrosstermBackend,
    style::{Style, Stylize},
    widgets::{Block, Borders, Tabs},
    Frame, Terminal,
};
use tokio::sync::mpsc;
use tui_textarea::{Input, TextArea};

use crate::app_project::models::ProjectModel;

use super::{
    app_components::SearchProjects,
    common::{
        component::{Drawable, Interactive, Painter},
        list_utils::{next, prev},
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
    project_name_input: TextArea<'static>,
    config: Rc<Config>,
    show_search_projects: bool,
    show_project_name_input: bool,
    pane_area: Rect,
}

impl App {
    pub fn new_from_project(
        project: ProjectModel,
        config: Rc<Config>,
        sender: mpsc::Sender<AppMessage>,
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
            pane_area: Rect::default(),
            project_name_input: input_element(),
            show_project_name_input: false,
            config,
        }
    }

    fn add_project(&mut self, project: ProjectModel, sender: mpsc::Sender<AppMessage>) {
        let mut pane = Pane::from_project(project, Rc::clone(&self.config), sender);
        pane.set_render_area(self.pane_area);
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
        let [tabs_area, pane_area] =
            Layout::vertical([Constraint::Length(2), Constraint::Fill(1)]).areas(viewport);
        self.tabs_area = tabs_area;
        self.pane_area = pane_area;

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
        } else if self.show_project_name_input {
            let area = center_area(
                frame.area(),
                Constraint::Length(3),
                Constraint::Percentage(40),
            );
            frame.render_widget(&self.project_name_input, area);
        }
    }

    pub fn handle_key(
        &mut self,
        key: KeyEvent,
        events: Rc<RefCell<Events<AppMessage>>>,
        terminal: Rc<RefCell<Terminal<CrosstermBackend<Stdout>>>>,
    ) {
        if self.show_search_projects {
            if let Some(effect) = self.search_projects.on_key(key, ()) {
                match effect {
                    super::app_components::SearchProjectsEffect::Submit(project) => {
                        self.add_project(project, events.borrow().sender());
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
                        self.project_name_input
                            .move_cursor(tui_textarea::CursorMove::End);
                        self.project_name_input.delete_line_by_head();
                        self.show_project_name_input = false;
                    }
                    keybinding::GlobalKeyAction::SubmitPopup => {
                        let renamed = self.project_name_input.lines()[0].to_owned();
                        self.current_pane_mut().unwrap().set_project_name(renamed);

                        // Save new name on mapping file
                        // {
                        //     let store = LocalStore::new();
                        // }

                        self.project_name_input
                            .move_cursor(tui_textarea::CursorMove::End);
                        self.project_name_input.delete_line_by_head();
                        self.show_project_name_input = false;
                    }
                    _ => {
                        self.project_name_input.input(Input::from(key));
                    }
                }
            } else {
                self.project_name_input.input(Input::from(key));
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
                            self.project_name_input.insert_str(&name);
                            self.show_project_name_input = true;
                        }
                    }
                },
                None => {
                    if let Some(pane) = self.current_pane_mut() {
                        pane.handle_key(key, events, terminal);
                    }
                }
            }
        }
    }
}

fn input_element() -> TextArea<'static> {
    let mut input = TextArea::default();
    input.set_block(
        Block::bordered()
            .title(format!(" {} ", "Project Name"))
            .border_style(Style::default().blue()),
    );

    input.set_cursor_line_style(Style::default());

    input
}
