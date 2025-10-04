use std::ops::Not;

use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Rect},
    style::{Style, Stylize},
    widgets::{Block, BorderType, Clear},
};

use crate::{
    app_project::models::{ContextEnv, EnvVariable},
    programs::tui::{
        common::{
            input::mode_input::ModeInput, list_utils, Interactive, UiComposedElement, UiElementV2,
        },
        config::{
            keybinding::{GlobalKeyAction, TableKeyAction},
            Config,
        },
        elements::{
            table::TableGrid,
            utils::{center_area, expand},
        },
    },
};

use super::{action::PaneAction, ElementFocus};

enum ContextOverlayType {
    Edit,
    New,
}

enum OverlayType {
    Context(ContextOverlayType),
    /// Show table of environments variables
    ShowDetails,
    None,
}

pub struct EnvironmentContexts {
    list: Vec<ContextTable>,
    /// Index for current visual context
    current: Option<usize>,
    input: ModeInput,
    overlay_type: OverlayType,
    render_area: Rect,
    // Table area
    details_area: Rect,
}

impl EnvironmentContexts {
    pub fn from_list(list: Vec<ContextEnv>) -> Self {
        let current = list.is_empty().not().then_some(0);

        let mut list_tables = Vec::new();
        for context in list {
            list_tables.push(ContextTable::new(context.name, context.variables));
        }

        Self {
            list: list_tables,
            current,
            input: ModeInput::new(""),
            render_area: Rect::default(),
            overlay_type: OverlayType::None,
            details_area: Rect::default(),
        }
    }

    pub fn get_model(&self) -> (Option<usize>, Vec<ContextEnv>) {
        let list = self
            .list
            .iter()
            .map(|table| ContextEnv {
                name: table.name.to_string(),
                variables: table.variables.clone(),
            })
            .collect();
        (self.current, list)
    }
}

impl<'params> UiComposedElement<'params> for EnvironmentContexts {
    type Params = (bool, &'params Config);

    fn set_area(&mut self, area: ratatui::prelude::Rect, viewport_area: ratatui::prelude::Rect) {
        self.render_area = area;
        self.details_area = center_area(
            viewport_area,
            Constraint::Percentage(40),
            Constraint::Percentage(40),
        );
        self.input.set_visual_width(area.width.saturating_sub(2));

        for table in &mut self.list {
            table.set_area(self.details_area, viewport_area);
        }
    }

    fn draw(&self, (is_focus, config): Self::Params, frame: &mut ratatui::Frame) {
        let block = Block::bordered()
            .border_style(Style::default().fg(if is_focus {
                config.theme.border_focus
            } else {
                config.theme.border
            }))
            .border_type(if is_focus {
                BorderType::Thick
            } else {
                BorderType::Plain
            });

        let area = block.inner(self.render_area);

        frame.render_widget(block, self.render_area);

        match self.current {
            Some(idx) => {
                let context = &self.list[idx];
                frame.render_widget(
                    expand(context.name.as_str(), " ", area.width as usize).blue(),
                    area,
                );
            }
            None => {
                frame.render_widget(
                    expand("No context selected", " ", area.width as usize)
                        .italic()
                        .dark_gray(),
                    area,
                );
            }
        }
    }

    fn draw_overlay(&self, (_focus, config): Self::Params, frame: &mut ratatui::Frame) {
        match &self.overlay_type {
            OverlayType::None => {}
            OverlayType::ShowDetails => {
                if let Some(context_table) = self.current.and_then(|idx| self.list.get(idx)) {
                    context_table.draw(config, frame);
                }
                // let area = center_area(
                //     frame.area(),
                //     Constraint::Percentage(50),
                //     Constraint::Percentage(40),
                // );
                // let block = Block::bordered()
                //     .title(format!(
                //         " {} ",
                //         self.list[self.current.unwrap()].name.to_uppercase()
                //     ))
                //     .border_type(ratatui::widgets::BorderType::Thick)
                //     .border_style(Style::default().fg(config.theme.border_focus));

                // let inner_area = block.inner(area);
                // frame.render_widget(Clear, area);
                // frame.render_widget(block, area);
                // // self.input.draw(inner_area, frame);
                // let table = TableGrid::new(["Name".blue(), "Value".blue()], [0.5, 0.5]).with_rows(
                //     self.list[self.current.unwrap()]
                //         .variables
                //         .iter()
                //         .map(|var| [var.name.as_str().red(), var.name.as_str().green()])
                //         .collect(),
                // );
                // frame.render_widget(table, inner_area);

                return;
            }
            OverlayType::Context(context_overlay_type) => {
                let title = match context_overlay_type {
                    ContextOverlayType::Edit => " Edit Context name ",
                    ContextOverlayType::New => " New Context ",
                };

                let area = center_area(
                    frame.area(),
                    Constraint::Length(3),
                    Constraint::Percentage(40),
                );
                let block = Block::bordered()
                    .title(title)
                    .border_style(Style::default().fg(config.theme.border_focus));
                let inner_area = block.inner(area);

                frame.render_widget(block, area);
                self.input.draw(inner_area, frame);
            }
        }
    }
}

impl<'params> Interactive<'params> for EnvironmentContexts {
    type Effect = PaneAction;

    type Params = (ElementFocus, &'params Config);

    fn handle_key(
        &mut self,
        (_focus, config): Self::Params,
        key: crossterm::event::KeyEvent,
    ) -> Self::Effect {
        match &self.overlay_type {
            OverlayType::ShowDetails => {
                let mut consumed = false;
                if let Some(action) = config.keymap.match_global_action(key) {
                    match action {
                        GlobalKeyAction::ClosePopup => {
                            // Check for display input, and avoid close de popup in this case
                            if !self
                                .current
                                .and_then(|idx| self.list.get_mut(idx))
                                .unwrap()
                                .is_visible_overlay()
                            {
                                self.overlay_type = OverlayType::None;
                                consumed = true;
                            }
                        }
                        _ => {}
                    }
                }

                if !consumed {
                    if let Some(table) = self.current.and_then(|idx| self.list.get_mut(idx)) {
                        table.handle_key(config, key)
                    }
                }
            }
            OverlayType::None => {
                let mut consumed = false;
                if let Some(action) = config.keymap.match_global_action(key) {
                    match action {
                        GlobalKeyAction::MoveLeft => {
                            self.current = list_utils::prev(self.current);
                            consumed = true;
                        }
                        GlobalKeyAction::MoveRight => {
                            self.current = list_utils::next(self.current, self.list.len());
                            consumed = true;
                        }
                        _ => {}
                    }
                }

                if !consumed {
                    if let KeyCode::Char(ch) = key.code {
                        match ch {
                            'e' => {
                                if self.current.is_some() {
                                    self.overlay_type =
                                        OverlayType::Context(ContextOverlayType::Edit);
                                    let context = &self.list[self.current.unwrap()];
                                    self.input.replace(&context.name);
                                }
                            }
                            'd' => {
                                // I should delete the current context
                                self.current =
                                    list_utils::delete_element(self.current, &mut self.list);
                            }
                            'n' => {
                                self.overlay_type = OverlayType::Context(ContextOverlayType::New);
                                self.input.clear();
                            }
                            _ => {}
                        }
                    }

                    if let KeyCode::Esc = key.code {
                        return PaneAction::RestoreFocus;
                    }

                    if let KeyCode::Enter = key.code {
                        // Show overlay for add table
                        if self.current.is_some() {
                            self.overlay_type = OverlayType::ShowDetails;
                        }
                    }
                }
            }
            OverlayType::Context(context_overlay_type) => match context_overlay_type {
                ContextOverlayType::Edit => {
                    let mut consumed = false;
                    if let Some(action) = config.keymap.match_global_action(key) {
                        match action {
                            GlobalKeyAction::ClosePopup => {
                                self.overlay_type = OverlayType::None;
                                consumed = true;
                            }
                            GlobalKeyAction::SubmitPopup => {
                                if self.input.txt().chars().next().is_some() {
                                    self.overlay_type = OverlayType::None;

                                    self.list[self.current.unwrap()].name =
                                        self.input.txt().to_string();
                                    self.input.clear();
                                }
                                consumed = true;
                            }
                            _ => {}
                        }
                    }

                    if !consumed {
                        self.input.handle_key(key);
                    }
                }
                ContextOverlayType::New => {
                    let mut consumed = false;
                    if let Some(action) = config.keymap.match_global_action(key) {
                        match action {
                            GlobalKeyAction::ClosePopup => {
                                self.overlay_type = OverlayType::None;
                                consumed = true;
                            }
                            GlobalKeyAction::SubmitPopup => {
                                if self.input.txt().chars().next().is_some() {
                                    self.overlay_type = OverlayType::None;
                                    let mut context_table =
                                        ContextTable::new(self.input.txt().to_string(), Vec::new());
                                    context_table.set_area(self.details_area, Rect::default());
                                    self.list.push(context_table);
                                    self.current = Some(self.list.len() - 1);
                                    self.input.clear();
                                }
                                consumed = true;
                            }
                            _ => {}
                        }
                    }

                    if !consumed {
                        self.input.handle_key(key);
                    }
                }
            },
        }

        PaneAction::Noop
    }
}

enum InputBehavior {
    New,
    Edit,
    Hidden,
}

impl InputBehavior {
    pub fn is_new(&self) -> bool {
        matches!(self, InputBehavior::New)
    }
}

struct ContextTable {
    name: String,
    variables: Vec<EnvVariable>,
    index_cell: Option<(usize, usize)>,
    input: ModeInput,
    input_behavior: InputBehavior,
    render_area: Rect,
}

impl ContextTable {
    pub fn new(name: String, variables: Vec<EnvVariable>) -> Self {
        let index_cell = if variables.is_empty() {
            None
        } else {
            Some((0, 0))
        };

        Self {
            name,
            variables,
            index_cell,
            input: ModeInput::new(""),
            input_behavior: InputBehavior::Hidden,
            render_area: Rect::default(),
        }
    }
}

// FIXME: I duplicating code from params_table :P
impl ContextTable {
    pub fn next_item(&mut self) {
        match self.index_cell {
            Some(idx) => {
                if idx.0 < self.variables.len() - 1 {
                    self.index_cell = Some((idx.0 + 1, idx.1));
                }
            }
            None => {
                if !self.variables.is_empty() {
                    self.index_cell = Some((0, 0))
                }
            }
        }
    }

    pub fn next_cell(&mut self) {
        if let Some((row, col)) = self.index_cell {
            let next_col = match col {
                0 => 1,
                _ => col,
            };

            self.index_cell = Some((row, next_col))
        }
    }

    pub fn previous_cell(&mut self) {
        if let Some((row, col)) = self.index_cell {
            let next_col = match col {
                1 => 0,
                _ => col,
            };

            self.index_cell = Some((row, next_col))
        }
    }

    pub fn previous_item(&mut self) {
        if let Some(idx) = self.index_cell {
            if idx.0 > 0 {
                self.index_cell = Some((idx.0 - 1, idx.1));
            }
        }
    }
}

impl<'params> UiComposedElement<'params> for ContextTable {
    type Params = &'params Config;

    fn set_area(&mut self, area: Rect, _viewport_area: Rect) {
        self.render_area = area;
        self.input.set_visual_width(area.width.saturating_sub(2));
    }

    fn draw(&self, config: Self::Params, frame: &mut ratatui::Frame) {
        frame.render_widget(Clear, self.render_area);

        let block = Block::bordered()
            .title(" Context Details ")
            .border_type(ratatui::widgets::BorderType::Thick)
            .border_style(Style::default().fg(config.theme.border_focus));

        let inner_area = block.inner(self.render_area);
        frame.render_widget(block, self.render_area);

        let table_rows = self
            .variables
            .iter()
            .map(|var| [var.name.as_str().into(), var.value.as_str().into()])
            .collect();

        let table = TableGrid::new(["Name".blue(), "Value".blue()], [0.5, 0.5])
            .with_rows(table_rows)
            .with_placeholder("There are no variables".into())
            .with_index(
                self.index_cell,
                Style::default()
                    .bg(config.theme.selection.bg)
                    .fg(config.theme.selection.fg),
            );
        frame.render_widget(table, inner_area);

        match self.input_behavior {
            InputBehavior::Hidden => {}
            _ => {
                let area = Rect {
                    y: inner_area.bottom() + 1,
                    height: 3,
                    ..self.render_area
                };

                let title = if self.input_behavior.is_new() {
                    " New Variable "
                } else {
                    " Edit param "
                };

                let block = Block::bordered()
                    .title(title)
                    .border_type(ratatui::widgets::BorderType::Thick)
                    .border_style(Style::default().fg(config.theme.border_focus));

                let input_area = block.inner(area);
                frame.render_widget(block, area);

                self.input.draw(input_area, frame);
            }
        }
    }
}

impl<'params> Interactive<'params> for ContextTable {
    type Effect = ();

    type Params = &'params Config;

    fn is_visible_overlay(&self) -> bool {
        matches!(
            self.input_behavior,
            InputBehavior::New | InputBehavior::Edit
        )
    }

    fn handle_key(
        &mut self,
        config: Self::Params,
        key: crossterm::event::KeyEvent,
    ) -> Self::Effect {
        match self.input_behavior {
            InputBehavior::Hidden => {
                let mut consumed = false;
                if let Some(action) = config.keymap.match_global_action(key) {
                    match action {
                        GlobalKeyAction::MoveDown => {
                            self.next_item();
                            consumed = true;
                        }
                        GlobalKeyAction::MoveUp => {
                            self.previous_item();
                            consumed = true;
                        }
                        GlobalKeyAction::MoveLeft => {
                            self.previous_cell();
                            consumed = true;
                        }
                        GlobalKeyAction::MoveRight => {
                            self.next_cell();
                            consumed = true;
                        }
                        _ => {}
                    }
                }

                if !consumed {
                    if let Some(action) = config.keymap.match_table_action(key) {
                        match action {
                            TableKeyAction::New => {
                                self.input_behavior = InputBehavior::New;
                            }
                            TableKeyAction::Delete => {
                                if let Some(cell) = self.index_cell {
                                    self.variables.remove(cell.0);
                                }
                            }
                            TableKeyAction::Edit => {
                                if let Some(idx) = self.index_cell {
                                    let variable = &self.variables[idx.0];
                                    let txt = match idx.1 {
                                        0 => variable.name.as_str(),
                                        1 => variable.value.as_str(),
                                        _ => unreachable!(),
                                    };
                                    self.input_behavior = InputBehavior::Edit;
                                    self.input.replace(txt);
                                }
                            }
                        }
                    }
                }
            }
            _ => {
                let mut consumed = false;
                if let Some(action) = config.keymap.match_global_action(key) {
                    match action {
                        GlobalKeyAction::ClosePopup => {
                            self.input_behavior = InputBehavior::Hidden;
                            consumed = true;
                        }
                        GlobalKeyAction::SubmitPopup => {
                            let txt = self.input.txt();

                            if txt.chars().next().is_some() {
                                if self.input_behavior.is_new() {
                                    self.variables.push(EnvVariable {
                                        name: txt.to_string(),
                                        value: String::new(),
                                    });
                                    self.index_cell = match self.index_cell {
                                        Some(idx) => Some(idx),
                                        None => Some((0, 0)),
                                    };
                                } else {
                                    let index_cell = self.index_cell.unwrap();
                                    let variable = &mut self.variables[index_cell.0];

                                    if index_cell.1 == 0 {
                                        variable.name = txt.to_string();
                                    } else {
                                        variable.value = txt.to_string();
                                    }
                                }
                                consumed = true;
                                self.input_behavior = InputBehavior::Hidden;
                                self.input.clear();
                            }
                        }
                        _ => {}
                    }
                }

                if !consumed {
                    self.input.handle_key(key);
                }
            }
        }
    }
}
