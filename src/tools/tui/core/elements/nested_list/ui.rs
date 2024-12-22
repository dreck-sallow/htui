use ratatui::{
    prelude::BlockExt,
    text::Text,
    widgets::{Block, List, Widget},
};

use super::cursor::NestedCursor;

pub struct NestedListSubItem<'a> {
    pub(crate) content: Text<'a>,
}

pub struct NestedListItem<'a> {
    pub(crate) content: Text<'a>,
    pub(crate) sub_items: Vec<NestedListSubItem<'a>>,
}

pub struct NestedListUi<'a> {
    pub(crate) block: Option<Block<'a>>,
    pub(crate) items: Vec<NestedListItem<'a>>,
    pub(crate) cursor: NestedCursor,
}

impl NestedListUi<'_> {
    fn visible_list(&self, height: usize) -> (NestedCursor, NestedCursor) {
        if self.items.is_empty() {
            return (NestedCursor::empty(), NestedCursor::empty());
        }

        let mut slice_height = 0;
        let mut init_cursor = NestedCursor::from(0);
        let mut end_cursor = NestedCursor::empty();

        for item in &self.items {
            let sub_items_count = item.sub_items.len();
            slice_height += 1 + sub_items_count; // parent + children length

            // mutate cursor
            end_cursor = NestedCursor::new(
                init_cursor.idx().map(|c| c + 1).or(Some(0)),
                item.sub_items
                    .is_empty()
                    .then_some(sub_items_count.saturating_sub(1)),
            );

            if slice_height >= height {
                if init_cursor <= self.cursor && self.cursor <= end_cursor {
                    // in range
                    return (init_cursor, end_cursor);
                } else {
                    init_cursor = end_cursor;
                    slice_height = 0
                }
            }
        }

        (NestedCursor::empty(), NestedCursor::empty())
    }
}

impl Widget for NestedListUi<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        self.block.render(area, buf);
        let inner_area = self.block.inner_if_some(area);

        if inner_area.is_empty() || self.items.is_empty() {
            return;
        }
    }
}
