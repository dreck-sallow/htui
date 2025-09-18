use std::{cell::RefCell, rc::Rc};

use arboard::Clipboard;
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Margin, Rect},
    style::{Style, Stylize},
    widgets::{Block, Clear},
};

use crate::{
    app_project::models::KeyValueParam,
    programs::tui::{
        common::{
            action_history::{ActionHistory, History, TrackAction},
            component::WithHistory,
            input::mode_input::ModeInput,
            InteractiveElement, UiElement,
        },
        config::{keybinding, Config},
        elements::{table::TableGrid, utils::center_area},
    },
};

pub struct TableParamState {
    items: Vec<KeyValueParam>,
    index_cell: Option<(usize, usize)>,
}

impl TableParamState {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            index_cell: None,
        }
    }

    pub fn set_items(&mut self, list: Vec<KeyValueParam>) {
        self.items = list;
        if !self.items.is_empty() {
            self.index_cell = Some((0, 0));
        }
    }

    pub fn next_item(&mut self) {
        match self.index_cell {
            Some(idx) => {
                if idx.0 < self.items.len() - 1 {
                    self.index_cell = Some((idx.0 + 1, idx.1));
                }
            }
            None => {
                if !self.items.is_empty() {
                    self.index_cell = Some((0, 0))
                }
            }
        }
    }

    pub fn next_cell(&mut self) {
        if let Some((row, col)) = self.index_cell {
            let next_col = match col {
                0 => 1,
                1 => 2,
                _ => col,
            };

            self.index_cell = Some((row, next_col))
        }
    }

    pub fn previous_cell(&mut self) {
        if let Some((row, col)) = self.index_cell {
            let next_col = match col {
                1 => 0,
                2 => 1,
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

    pub fn get_item(&self, idx: usize) -> Option<&KeyValueParam> {
        self.items.get(idx)
    }

    pub fn insert_param(&mut self, idx: usize, item: KeyValueParam) {
        self.items.insert(idx, item);

        if self.index_cell.is_none() {
            self.index_cell = Some((0, 0));
        }
    }

    pub fn remove_param(&mut self, idx: usize) -> Option<KeyValueParam> {
        if self.items.get(idx).is_some() {
            if idx == 0 {
                if self.items.len() <= 1 {
                    self.index_cell = None;
                }
            } else if idx == self.items.len() - 1 {
                let current_index = self.index_cell.unwrap();
                self.index_cell = Some((idx - 1, current_index.1));
            }

            Some(self.items.remove(idx))
        } else {
            None
        }
    }

    pub fn edit_item<F: FnMut(&mut KeyValueParam)>(&mut self, idx: usize, mut f: F) {
        if let Some(item) = self.items.get_mut(idx) {
            f(item);
        }
    }
}

pub struct ParamsTable {
    state: TableParamState,
    input: ModeInput,
    show_popup: bool,
    render_area: Rect,
    _history: ActionHistory<TableParamsAction>,
}

impl ParamsTable {
    pub fn new() -> Self {
        Self {
            render_area: Rect::default(),
            state: TableParamState::new(),
            input: ModeInput::new(""),
            show_popup: false,
            _history: ActionHistory::new(),
        }
    }

    pub fn len_items(&self) -> usize {
        self.state.items.len()
    }

    pub fn set_state(&mut self, params: Vec<KeyValueParam>) {
        self.clean_lines();
        self.state.set_items(params);
        self._history.clean();
    }

    pub fn get_data(&self) -> Vec<KeyValueParam> {
        self.state.items.clone()
    }

    pub fn clean_lines(&mut self) {
        self.input.clear();
    }

    pub fn cell_text(&self) -> Option<&str> {
        self.state.index_cell.and_then(|(row_i, col_i)| {
            let row = &self.state.items[row_i];
            match col_i {
                0 => Some(if row.enable { "true" } else { "false" }),
                1 => Some(row.key.as_ref()),
                2 => Some(row.value.as_ref()),
                _ => None,
            }
        })
    }

    pub fn is_editing(&self) -> bool {
        self.show_popup
    }
}

impl UiElement for ParamsTable {
    type Params = Rc<Config>;

    fn set_area(&mut self, area: Rect) {
        self.render_area = area;
        self.input.set_area(
            center_area(
                self.render_area,
                Constraint::Length(3),
                Constraint::Percentage(50),
            )
            .inner(Margin::new(1, 1)),
        );
    }

    fn draw(&self, config: Self::Params, frame: &mut ratatui::Frame) {
        let rows = self
            .state
            .items
            .iter()
            .map(|param| {
                [
                    (if param.enable { "yes" } else { "no" }).into(),
                    param.key.as_str().into(),
                    param.value.as_str().into(),
                ]
            })
            .collect();

        let table_grid = TableGrid::new(
            ["Enabled".blue(), "key".blue(), "Value".blue()],
            [0.2, 0.4, 0.4],
        )
        .with_index(
            self.state.index_cell,
            Style::default()
                .fg(config.theme.selection.fg)
                .bg(config.theme.selection.bg),
        )
        .with_placeholder("No Items".italic().dark_gray())
        .with_rows(rows);

        frame.render_widget(table_grid, self.render_area);
    }

    fn draw_overlay(&self, config: Self::Params, frame: &mut ratatui::Frame) {
        if self.show_popup {
            let area = center_area(
                self.render_area,
                Constraint::Length(3),
                Constraint::Percentage(50),
            );
            let block = Block::bordered()
                .title("| Edit |")
                .border_style(Style::default().fg(config.theme.border_focus))
                .border_type(ratatui::widgets::BorderType::Thick);

            frame.render_widget(Clear, area);
            frame.render_widget(block, area);
            self.input.draw((), frame);
        }
    }
}

impl<'params> InteractiveElement<'params> for ParamsTable {
    type Params = (Rc<Config>, Rc<RefCell<Clipboard>>);

    fn handle_key(&mut self, (config, clipboard): Self::Params, key: crossterm::event::KeyEvent) {
        if self.show_popup {
            let is_consumed = match config.keymap.match_global_action(key) {
                Some(action) => match action {
                    keybinding::GlobalKeyAction::ClosePopup if self.input.mode().is_read_mode() => {
                        self.show_popup = false;
                        self.clean_lines();
                        true
                    }
                    keybinding::GlobalKeyAction::SubmitPopup => {
                        if let Some((row, col)) = self.state.index_cell {
                            let txt = self.input.txt().to_owned();
                            match col {
                                1 => {
                                    self._history.apply(
                                        TableParamsAction::EditKey(row, txt),
                                        &mut self.state,
                                    );
                                    self.clean_lines();
                                }
                                2 => {
                                    self._history.apply(
                                        TableParamsAction::EditValue(row, txt),
                                        &mut self.state,
                                    );
                                    self.clean_lines();
                                }
                                _ => {}
                            }
                        }

                        self.show_popup = false;
                        self.input.mode();
                        self.clean_lines();
                        true
                    }
                    _ => false,
                },
                None => key.code == KeyCode::Enter,
            };

            if !is_consumed {
                self.input.handle_key(key);
            }
        } else {
            let is_consumed = config
                .keymap
                .match_global_action(key)
                .map_or(false, |action| match action {
                    keybinding::GlobalKeyAction::MoveDown => {
                        self.state.next_item();
                        true
                    }
                    keybinding::GlobalKeyAction::MoveUp => {
                        self.state.previous_item();
                        true
                    }
                    keybinding::GlobalKeyAction::MoveLeft => {
                        self.state.previous_cell();
                        true
                    }
                    keybinding::GlobalKeyAction::MoveRight => {
                        self.state.next_cell();
                        true
                    }
                    keybinding::GlobalKeyAction::CopyToClipboard => {
                        if let Some(txt) = self.cell_text() {
                            clipboard.borrow_mut().set_text(txt).unwrap();
                        }
                        true
                    }
                    _ => false,
                });

            if !is_consumed {
                if let Some(action) = config.keymap.match_table_action(key) {
                    match action {
                        keybinding::TableKeyAction::New => {
                            self._history.apply(
                                TableParamsAction::AddItem(KeyValueParam::default()),
                                &mut self.state,
                            );
                        }
                        keybinding::TableKeyAction::Delete => {
                            if let Some((row, _)) = self.state.index_cell {
                                self._history
                                    .apply(TableParamsAction::RemoveItem(row), &mut self.state);
                            };
                        }
                        keybinding::TableKeyAction::Edit => {
                            if let Some((row, col)) = self.state.index_cell {
                                match col {
                                    0 => {
                                        let key_value = self.state.get_item(row).unwrap();
                                        self._history.apply(
                                            TableParamsAction::MarkApply(row, !key_value.enable),
                                            &mut self.state,
                                        );
                                    }
                                    1 => {
                                        self.clean_lines();
                                        let key_value = self.state.get_item(row).unwrap();
                                        self.show_popup = true;
                                        self.input.replace(&key_value.key);
                                    }
                                    2 => {
                                        self.clean_lines();
                                        let key_value = self.state.get_item(row).unwrap();
                                        self.show_popup = true;
                                        self.input.replace(&key_value.value);
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

enum TableParamsAction {
    AddItem(KeyValueParam),
    InsertItem(usize, KeyValueParam),
    RemoveItem(usize),
    MarkApply(usize, bool),
    EditKey(usize, String),
    EditValue(usize, String),
}

impl TrackAction for TableParamsAction {
    type State = TableParamState;

    fn apply(&self, state: &mut Self::State) -> Option<Self>
    where
        Self: Sized,
    {
        match self {
            TableParamsAction::AddItem(key_value_param) => {
                let idx = state.items.len();
                state.insert_param(idx, key_value_param.clone());
                Some(Self::RemoveItem(idx))
            }
            TableParamsAction::InsertItem(idx, key_value_param) => {
                state.insert_param(*idx, key_value_param.clone());
                Some(Self::RemoveItem(*idx))
            }
            TableParamsAction::RemoveItem(idx) => {
                let param = state.remove_param(*idx).unwrap();

                Some(Self::InsertItem(*idx, param))
            }
            TableParamsAction::MarkApply(idx, flag) => {
                state.edit_item(*idx, |itm| {
                    itm.enable = *flag;
                });

                Some(Self::MarkApply(*idx, !flag))
            }
            TableParamsAction::EditKey(idx, new_key) => {
                let previous_key = state.get_item(*idx).unwrap().key.to_owned();
                state.edit_item(*idx, |itm| itm.key = new_key.clone());
                Some(Self::EditKey(*idx, previous_key))
            }
            TableParamsAction::EditValue(idx, new_value) => {
                let previous_value = state.get_item(*idx).unwrap().value.to_owned();
                state.edit_item(*idx, |itm| itm.value = new_value.clone());
                Some(Self::EditValue(*idx, previous_value))
            }
        }
    }
}

impl WithHistory for ParamsTable {
    fn undo(&mut self) {
        self._history.undo(&mut self.state);
    }

    fn redo(&mut self) {
        self._history.redo(&mut self.state);
    }
}
