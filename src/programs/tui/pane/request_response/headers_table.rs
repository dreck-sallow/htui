use ratatui::style::{Style, Stylize};

use crate::{
    programs::tui::elements::table::{TableGrid, TableGridIndex},
    store::models::KeyValueParam,
};

pub struct HeadersTable {
    items: Vec<KeyValueParam>,
    index: TableGridIndex,
}

impl HeadersTable {
    pub fn new(items: Vec<KeyValueParam>) -> Self {
        let idx = TableGridIndex::new(items.len());
        Self {
            items: items,
            index: idx,
        }
    }

    pub fn table_ui(&self) -> TableGrid<'_, '_, 3> {
        let rows = self
            .items
            .iter()
            .map(|param| {
                [
                    if param.enable { "yes" } else { "no" }.into(),
                    param.key.as_str().into(),
                    param.value.as_str().into(),
                ]
            })
            .collect();

        TableGrid::new(
            ["Enable".blue(), "Header name".blue(), "Header value".blue()],
            [0.1, 0.4, 0.5],
        )
        .with_index(
            self.index.index(),
            Style::default().on_light_red().dark_gray(),
        )
        .with_placeholder("No items".italic().dark_gray())
        .with_rows(rows)
    }
}
