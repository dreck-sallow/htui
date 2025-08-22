use std::rc::Rc;

use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Margin, Rect},
    style::{Style, Stylize},
    text::Span,
    widgets::{Block, Clear, Widget},
};
use tui_textarea::{CursorMove, Input, TextArea};

use crate::{
    app_project::models::KeyValueParam,
    programs::tui::{
        common::{
            action_history::{ActionHistory, History, TrackAction},
            component::{Drawable, Interactive, WithHistory},
        },
        config::{keybinding, Config},
        elements::utils::center_area,
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
                self.index_cell = None;
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

pub struct TableParams {
    render_area: Rect,
    state: TableParamState,
    input: TextArea<'static>,
    /// For show the input for editing a field
    show_popup: bool,
    config: Rc<Config>,
    _history: ActionHistory<TableParamsAction>,
}

impl TableParams {
    pub fn new(config: Rc<Config>) -> Self {
        let mut input = TextArea::default();
        input.set_block(
            Block::bordered()
                .title("| Edit |")
                .border_style(Style::default().fg(config.theme.border_focus))
                .border_type(ratatui::widgets::BorderType::Thick),
        );
        input.set_cursor_line_style(Style::default());

        Self {
            render_area: Rect::default(),
            state: TableParamState::new(),
            input,
            show_popup: false,
            config,
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
        self.input.move_cursor(CursorMove::End);
        self.input.delete_line_by_head();
    }

    pub fn is_editing(&self) -> bool {
        self.show_popup
    }
}

impl Drawable for TableParams {
    type Params = ();

    fn set_area(&mut self, _area: Rect) {
        self.render_area = _area.inner(Margin::new(1, 0));
    }

    fn draw<'a: 'painter, 'painter>(
        &'a self,
        painter: &mut crate::programs::tui::common::component::Painter<'painter>,
        _params: Self::Params,
    ) {
        painter.render(move |frame| {
            let items = self
                .state
                .items
                .iter()
                .map(|itm| {
                    let apply_label = if itm.enable { "yes" } else { "no" };
                    [
                        Span::raw(apply_label),
                        Span::from(&itm.key),
                        Span::from(&itm.value),
                    ]
                })
                .collect();
            let table = TableParamsUi::new(items)
                .title_style(Style::default().gray().blue())
                .index_style(Style::default().on_light_blue().dark_gray())
                .index_cell(self.state.index_cell);

            frame.render_widget(table, self.render_area);
        });

        if self.show_popup {
            painter.render_last(|frame| {
                let area = center_area(
                    self.render_area,
                    Constraint::Length(3),
                    Constraint::Percentage(50),
                );

                frame.render_widget(Clear, area);
                frame.render_widget(&self.input, area);
            });
        }
    }
}

impl Interactive for TableParams {
    type Effect = ();
    type Params = ();

    fn on_key(
        &mut self,
        key: crossterm::event::KeyEvent,
        _params: Self::Params,
    ) -> Option<Self::Effect> {
        if self.show_popup {
            let is_consumed = match self.config.keymap.match_global_action(key) {
                Some(action) => match action {
                    keybinding::GlobalKeyAction::ClosePopup => {
                        self.show_popup = false;
                        self.clean_lines();
                        true
                    }
                    keybinding::GlobalKeyAction::SubmitPopup => {
                        if let Some((row, col)) = self.state.index_cell {
                            let txt = self.input.lines()[0].to_owned();
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
                        self.clean_lines();
                        true
                    }
                    _ => false,
                },
                None => key.code == KeyCode::Enter,
            };

            if !is_consumed {
                self.input.input(Input::from(key));
            }
        } else {
            let is_consumed = self
                .config
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
                    _ => false,
                });

            if !is_consumed {
                if let Some(action) = self.config.keymap.match_table_action(key) {
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
                                        self.input.insert_str(&key_value.key);
                                    }
                                    2 => {
                                        self.clean_lines();
                                        let key_value = self.state.get_item(row).unwrap();
                                        self.show_popup = true;
                                        self.input.insert_str(&key_value.value);
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }
        None
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

impl WithHistory for TableParams {
    fn undo(&mut self) {
        self._history.undo(&mut self.state);
    }

    fn redo(&mut self) {
        self._history.redo(&mut self.state);
    }
}

/// Internal table for ui
pub struct TableParamsUi<'text> {
    header: [&'static str; 3],
    key_values: Vec<[Span<'text>; 3]>,
    title_style: Style,
    index_style: Style,
    index_cell: Option<(usize, usize)>,
}

impl<'text> TableParamsUi<'text> {
    pub fn new(items: Vec<[Span<'text>; 3]>) -> Self {
        let items_len = items.len();
        Self {
            header: ["Enable", "Key", "Value"],
            key_values: items,
            title_style: Style::default(),
            index_style: Style::default(),
            index_cell: if items_len > 0 { Some((0, 0)) } else { None },
        }
    }

    pub fn title_style(mut self, style: Style) -> Self {
        self.title_style = style;
        self
    }

    pub fn index_style(mut self, style: Style) -> Self {
        self.index_style = style;
        self
    }

    pub fn index_cell(mut self, index: Option<(usize, usize)>) -> Self {
        self.index_cell = index;
        self
    }
}

impl Widget for TableParamsUi<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        if area.is_empty() {
            return;
        }

        let header_area = Rect { height: 1, ..area };
        let content_area = Rect {
            y: area.y + 1,
            ..area
        };

        let colum_widths = {
            let missing_width = (area.width - 2) - 6;
            let key_width = ((missing_width as f32) * 0.4) as u16;

            [6, key_width, missing_width - key_width]
        };
        // Draw the header (titles)

        let mut left = header_area.left();
        for (i, title) in self.header.iter().enumerate() {
            let width = colum_widths[i];
            buf.set_stringn(
                left,
                header_area.top(),
                title,
                width as usize,
                self.title_style,
            );

            left += width + 1;
        }

        // If no items, render a placeholder

        if self.key_values.is_empty() {
            let msg = "No items";
            buf.set_stringn(
                content_area.left(),
                content_area.top(),
                msg,
                msg.len(),
                Style::default().gray().italic(),
            );
        } else {
            // Draw the list table content
            // get the visible page
            let (page_start, page_end) = {
                let in_page = |start_i: usize| {
                    let height = content_area.height;
                    let mut acc_height = 0;

                    let mut end_i = start_i;

                    for _ in &self.key_values[start_i..] {
                        // Handle multilines?
                        acc_height += 1;

                        if acc_height > height {
                            break;
                        }

                        end_i += 1;
                    }

                    (start_i, end_i)
                };

                match self.index_cell {
                    Some((row_i, _)) => loop {
                        let idx = in_page(0);
                        if row_i >= idx.0 && row_i <= idx.1 {
                            break idx;
                        }
                    },
                    None => in_page(0),
                }
            };

            let mut top = content_area.top();
            let mut left = content_area.left();

            let check_idx = |n: (usize, usize)| {
                if let Some(idx) = self.index_cell {
                    let from_start_idx = idx.0 - page_start;
                    n == (from_start_idx, idx.1)
                } else {
                    false
                }
            };

            for (row, key_value) in self.key_values[page_start..page_end].iter().enumerate() {
                for (col, text) in key_value.iter().enumerate() {
                    let width = colum_widths[col];
                    if text.width() == 0 {
                        // render a placeholder!
                        buf.set_string(left, top, "-", Style::default().italic().dark_gray());
                    } else {
                        buf.set_span(left, top, text, width);
                    }

                    if check_idx((row, col)) {
                        buf.set_style(
                            Rect {
                                x: left,
                                y: top,
                                width,
                                height: 1, // TODO: change for multiline
                            },
                            self.index_style,
                        );
                    }

                    left += width + 1;
                }

                top += 1;
                left = content_area.left();
            }
        }
    }
}
