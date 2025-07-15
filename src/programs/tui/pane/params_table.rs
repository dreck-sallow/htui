use crossterm::event::KeyCode;
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::Span,
    widgets::Widget,
};
use tui_textarea::TextArea;

use crate::{
    programs::tui::common::{
        action_history::{ActionHistory, History, TrackAction},
        component::{Drawable, Interactive},
    },
    store::models::KeyValueParam,
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

    pub fn previous_item(&mut self) {
        if let Some(idx) = self.index_cell {
            if idx.0 > 0 {
                self.index_cell = Some((idx.0 - 1, idx.1));
            }
        }
    }

    pub fn insert_param(&mut self, idx: usize, item: KeyValueParam) {
        self.items.insert(idx, item);

        if let None = self.index_cell {
            self.index_cell = Some((0, 0));
        }
    }

    pub fn remove_param(&mut self, idx: usize) -> Option<KeyValueParam> {
        if self.items.get(idx).is_some() {
            if idx == 0 {
                self.index_cell = None;
            } else if idx == self.items.len() - 1 {
                let current_index = self.index_cell.unwrap();
                self.index_cell = Some((current_index.0 - 1, current_index.1));
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
    _history: ActionHistory<TableParamsAction>,
}

impl TableParams {
    pub fn new() -> Self {
        Self {
            render_area: Rect::default(),
            state: TableParamState::new(),
            input: TextArea::new(Vec::new()),
            _history: ActionHistory::new(),
        }
    }
}

impl Drawable for TableParams {
    type Params = ();

    fn set_area(&mut self, _area: Rect) {
        self.render_area = _area;
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
                .map(|itm| [Span::raw("*"), Span::from(&itm.key), Span::from(&itm.value)])
                .collect();
            let table = ParamsTable::new(items)
                .title_style(Style::default().on_blue().white())
                .index_style(Style::default().on_light_blue().white())
                .index_cell(self.state.index_cell.clone());

            frame.render_widget(table, self.render_area);
        });
    }
}

impl Interactive for TableParams {
    type Effect = TableParamsEffect;

    fn on_key(&mut self, key: crossterm::event::KeyEvent) -> Option<Self::Effect> {
        match key.code {
            KeyCode::Char('n') => self._history.apply(
                TableParamsAction::AddItem(KeyValueParam::default()),
                &mut self.state,
            ),
            _ => {}
        }
        None
    }
}

enum TableParamsAction {
    AddItem(KeyValueParam),
    InsertItem(usize, KeyValueParam),
    RemoveItem(usize),
    MarkApply(usize, bool),
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
                    itm.apply = *flag;
                });

                Some(Self::MarkApply(*idx, !flag))
            }
        }
    }
}

pub enum TableParamsEffect {
    NextFocus,
    PreviousFocus,
}

pub struct ParamsTable<'text> {
    header: [&'static str; 3],
    key_values: Vec<[Span<'text>; 3]>,
    title_style: Style,
    index_style: Option<Style>,
    index_cell: Option<(usize, usize)>,
}

impl<'text> ParamsTable<'text> {
    pub fn new(items: Vec<[Span<'text>; 3]>) -> Self {
        let items_len = items.len();
        Self {
            header: ["Enable", "Key", "Value"],
            key_values: items,
            title_style: Style::default(),
            index_style: None,
            index_cell: if items_len > 0 { Some((0, 0)) } else { None },
        }
    }

    pub fn title_style(mut self, style: Style) -> Self {
        self.title_style = style;
        self
    }

    pub fn index_style(mut self, style: Style) -> Self {
        self.index_style = Some(style);
        self
    }

    pub fn index_cell(mut self, index: Option<(usize, usize)>) -> Self {
        self.index_cell = index;
        self
    }
}

impl<'a> Widget for ParamsTable<'a> {
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
            let missing_width = area.width - 6;
            let key_width = ((missing_width as f32) * 0.4) as u16;

            [6, key_width, missing_width - key_width]
        };
        // Draw the header (titles)

        for (i, title) in self.header.iter().enumerate() {
            let width = colum_widths[i];
            buf.set_stringn(
                header_area.left() + (i as u16 * width),
                header_area.top(),
                title,
                width as usize,
                self.title_style,
            );
        }

        // If no items, render a placeholder

        if self.key_values.is_empty() {
            let msg = "No items";
            buf.set_stringn(
                content_area.left(),
                content_area.top(),
                msg,
                msg.len(),
                Style::default().gray(),
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

            for key_value in &self.key_values[page_start..(page_end + 1)] {
                for (i, text) in key_value.iter().enumerate() {
                    let width = colum_widths[i];
                    buf.set_span(i as u16 * width, top, text, width);
                }

                top += 1;
            }
        }
    }
}
