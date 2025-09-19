use std::{collections::HashSet, ops::Not, rc::Rc};

use ratatui::{
    layout::{Constraint, Rect},
    style::{Style, Stylize},
    text::Span,
    widgets::{Block, Clear},
};

use crate::{
    app_project::{
        models::ProjectModel,
        paths::ProjectPaths,
        store::{LocalStore, Store, StoreProjectItem},
    },
    programs::tui::{
        common::{
            list_utils::{next, prev},
            InteractiveElementEff, UiElement,
        },
        config::{keybinding, Config},
        elements::utils::center_area,
    },
};

pub struct SearchProjects {
    store: LocalStore<ProjectPaths>,
    list: Vec<StoreProjectItem>,
    selected: Option<usize>,
    filtereds: Vec<usize>,
    /// List of ProjectId, for no show to user
    // search_input: TextArea<'static>,
    config: Rc<Config>,
}

impl SearchProjects {
    pub fn new(config: Rc<Config>) -> Self {
        // let mut input = TextArea::new(vec![]);
        // input.set_cursor_line_style(Style::default());

        Self {
            store: LocalStore::new(),
            list: Vec::new(),
            selected: None,
            filtereds: Vec::new(),
            // search_input: input,
            config,
        }
    }

    pub fn search(&mut self, current_openeds: HashSet<String>) {
        let non_openeds = {
            let mut list = Vec::new();
            for project in self.store.project_list().unwrap() {
                if !current_openeds.contains(&project.id) {
                    list.push(project);
                }
            }
            list
        };

        // TODO: Handle the fs reads errors
        self.selected = non_openeds.is_empty().not().then_some(0);
        self.filtereds = non_openeds.iter().enumerate().map(|(i, _)| i).collect();
        self.list = non_openeds;
    }

    pub fn list_page(&self, height: u16) -> Vec<Span> {
        let page = self.selected.unwrap_or(0);

        let mut list = Vec::new();

        if let Some(indexes) = self
            .filtereds
            .chunks(height as usize)
            .skip(page / (height as usize))
            .next()
        {
            for idx in indexes {
                list.push(Span::from(&self.list[*idx].name));
            }
        }

        list
    }
}

impl UiElement for SearchProjects {
    type Params = ();

    fn set_area(&mut self, _area: Rect) {}

    fn draw(&self, _params: Self::Params, frame: &mut ratatui::Frame) {
        let height = {
            let content_size = if self.filtereds.is_empty() {
                1 // placeholder
            } else {
                8.min(self.filtereds.len())
            };

            2 + content_size
        };
        let area = center_area(
            frame.area(),
            Constraint::Length(height as u16),
            Constraint::Percentage(30),
        );

        let block = Block::bordered()
            .title("| Search Project |")
            .title_alignment(ratatui::layout::Alignment::Center)
            .border_style(self.config.theme.border_focus)
            .border_type(ratatui::widgets::BorderType::Thick);

        let mut inner_area = block.inner(area);

        frame.render_widget(Clear, area);

        frame.render_widget(block, area);

        if self.filtereds.is_empty() {
            frame.render_widget(Span::from("No projects to select").italic(), inner_area);
        } else {
            for (idx, itm) in self.list_page(inner_area.height).iter().enumerate() {
                // TODO: reuse the inner_area
                let item_area = Rect {
                    height: 1,
                    ..inner_area
                };
                frame.render_widget(itm, item_area);

                let style = if self.selected.map(|i| i == idx).unwrap_or(false) {
                    Style::default()
                        .fg(self.config.theme.dropdown_highlight.fg)
                        .bg(self.config.theme.dropdown_highlight.bg)
                } else {
                    Style::default().fg(self.config.theme.dropdown.fg).bg(self
                        .config
                        .theme
                        .dropdown
                        .bg)
                };

                frame.buffer_mut().set_style(item_area, style);

                inner_area.y += 1;
            }
        }
    }
}

impl<'p> InteractiveElementEff<'p> for SearchProjects {
    type Effect = Option<SearchProjectsEffect>;

    type Params = ();

    fn handle_key(
        &mut self,
        _params: Self::Params,
        key: crossterm::event::KeyEvent,
    ) -> Self::Effect {
        if let Some(action) = self.config.keymap.match_global_action(key) {
            match action {
                keybinding::GlobalKeyAction::MoveDown => {
                    self.selected = next(self.selected, self.filtereds.len());
                }
                keybinding::GlobalKeyAction::MoveUp => {
                    self.selected = prev(self.selected);
                }
                keybinding::GlobalKeyAction::ClosePopup => {
                    self.list = Vec::new();
                    self.filtereds = Vec::new();
                    self.selected = None;
                    return Some(SearchProjectsEffect::Hidden);
                }
                keybinding::GlobalKeyAction::SubmitPopup => {
                    if let Some(i) = self.selected {
                        let idx = self.filtereds[i];
                        if let Ok(model) = self.store.get_project(self.list[idx].id.clone()) {
                            return Some(SearchProjectsEffect::Submit(model));
                        }
                    }
                }
                _ => {}
            }
        }
        None
    }
}

pub enum SearchProjectsEffect {
    Submit(ProjectModel),
    Hidden,
}
