use ratatui::style::{Style, Stylize};

use crate::programs::tui::{
    config::Theme,
    elements::table::{TableGrid, TableGridIndex},
};

pub struct HeadersTable {
    items: Vec<(String, String)>,
    index: TableGridIndex,
}

impl HeadersTable {
    pub fn new(items: Vec<(String, String)>) -> Self {
        let idx = TableGridIndex::new(items.len());
        Self { items, index: idx }
    }

    pub fn len_items(&self) -> usize {
        self.items.len()
    }

    pub fn move_col_idx(&mut self, is_next: bool) {
        if is_next {
            self.index.next_col(2);
        } else {
            self.index.previous_col();
        }
    }

    pub fn move_row_idx(&mut self, is_next: bool) {
        if is_next {
            self.index.next_row(self.items.len());
        } else {
            self.index.previous_row();
        }
    }

    pub fn replace<Items: Into<Vec<(String, String)>>>(&mut self, items: Items) {
        let items = items.into();
        self.index = TableGridIndex::new(items.len());
        self.items = items;
    }

    pub fn table_ui(&self, theme: &Theme) -> TableGrid<'_, '_, 2> {
        let rows = self
            .items
            .iter()
            .map(|(key, value)| [key.as_str().into(), value.as_str().into()])
            .collect();

        TableGrid::new(["Header name".blue(), "Header value".blue()], [0.4, 0.5])
            .with_index(
                self.index.index(),
                Style::default()
                    .fg(theme.selection.fg)
                    .bg(theme.selection.bg),
            )
            .with_placeholder("No items".italic().dark_gray())
            .with_rows(rows)
    }
}
