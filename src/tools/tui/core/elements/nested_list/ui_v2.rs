use ratatui::{
    layout::Rect,
    prelude::BlockExt,
    style::Style,
    text::Text,
    widgets::{Block, Widget},
};

use super::cursor::NestedCursor;

/// Type used for manage flatten ´NestedListItem´ with its sub items
pub enum NestedListItem<'a> {
    /// Variant used for items level
    L1 { text: Text<'a> },

    /// Variant used for nested level
    L2 { text: Text<'a> },
}

impl<'a> NestedListItem<'a> {
    pub fn height(&self) -> usize {
        match self {
            NestedListItem::L1 { text, .. } => text.height(),
            NestedListItem::L2 { text } => text.height(),
        }
    }

    pub fn text(&self) -> &Text<'a> {
        match self {
            NestedListItem::L1 { text, .. } => text,
            NestedListItem::L2 { text } => text,
        }
    }

    pub fn is_l1(&self) -> bool {
        match self {
            NestedListItem::L1 { .. } => true,
            NestedListItem::L2 { .. } => false,
        }
    }

    pub fn is_l2(&self) -> bool {
        match self {
            NestedListItem::L1 { .. } => false,
            NestedListItem::L2 { .. } => true,
        }
    }
}

pub struct NestedList<'a> {
    pub block: Option<Block<'a>>,
    pub items: Vec<NestedListItem<'a>>,
    cursor: NestedCursor,
}

impl<'a> NestedList<'a> {
    pub fn new(items: Vec<NestedListItem<'a>>) -> Self {
        Self {
            items,
            block: None,
            cursor: NestedCursor::empty(),
        }
    }

    pub fn with_block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn with_cursor(mut self, cursor: NestedCursor) -> Self {
        self.cursor = cursor;
        self
    }

    /// Method used for generating the visible list
    fn visible_list(&self, height: usize) -> (NestedCursor, NestedCursor) {
        if self.items.is_empty() || self.cursor == NestedCursor::empty() {
            return (NestedCursor::empty(), NestedCursor::empty());
        }

        // compute visible list
        let mut current_height = 0;
        let mut init_cursor = NestedCursor::from(0);
        let mut end_cursor = NestedCursor::empty();

        for item in &self.items {
            if current_height + item.height() > height
                && init_cursor <= self.cursor
                && self.cursor <= end_cursor
            {
                return (init_cursor, end_cursor);
            }

            if item.is_l1() {
                end_cursor.add_idx(1);
                end_cursor.set_sub_idx(None);
            } else {
                end_cursor.add_sub_idx(1);
            }

            if current_height + item.height() <= height {
                current_height += item.height();
            } else {
                current_height = item.height();
                init_cursor = end_cursor.clone();
            }
        }

        (init_cursor, end_cursor)
    }

    fn slice_in_range(
        &self,
        range: (NestedCursor, NestedCursor),
    ) -> (&[NestedListItem<'a>], Option<usize>) {
        let (start_cursor, end_cursor) = range;

        if start_cursor <= end_cursor {
            let (mut start_index, mut end_index) = (0, 0);
            let mut selected_index = None;

            let mut current_cursor = NestedCursor::empty();

            for (i, item) in self.items.iter().enumerate() {
                if item.is_l2() {
                    current_cursor.add_sub_idx(1);
                } else {
                    current_cursor.add_idx(1);
                    current_cursor.set_sub_idx(None);
                }

                if current_cursor == self.cursor {
                    selected_index = Some(i - start_index);
                }

                if current_cursor == start_cursor {
                    start_index = i;
                } else if current_cursor == end_cursor {
                    end_index = i;
                    break;
                }
            }
            return (
                &self.items[start_index..(end_index).saturating_add(1)],
                selected_index,
            );
        }

        (&[], None)
    }
}

impl Widget for NestedList<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        self.block.render(area, buf);
        let inner_area = self.block.inner_if_some(area);

        if inner_area.is_empty() || self.items.is_empty() {
            return;
        }

        let range = self.visible_list(inner_area.height as usize);

        let mut current_height = 0;

        let (items, selected) = self.slice_in_range(range);

        for (i, item) in items.iter().enumerate() {
            let selected_itm = selected.map_or(false, |idx| idx == i);

            let left_symbol = {
                if item.is_l1() {
                    if selected_itm {
                        "> "
                    } else {
                        "  "
                    }
                } else if selected_itm {
                    "  > "
                } else {
                    "    "
                }
            };

            let area = Rect {
                y: inner_area.top() + current_height as u16,
                x: inner_area.left() + left_symbol.len() as u16,
                height: item.height() as u16,
                width: inner_area.width.saturating_sub(left_symbol.len() as u16),
            };

            item.text().clone().render(area, buf);
            current_height += item.height();

            buf.set_stringn(
                inner_area.x,
                current_height as u16,
                left_symbol,
                inner_area.width as usize,
                Style::default(),
            );
        }
    }
}
