use ratatui::{
    layout::Rect,
    prelude::BlockExt,
    text::Text,
    widgets::{Block, Widget},
};

use super::cursor::NestedCursor;

// #[derive(Clone)]
pub struct NestedListSubItemUi<'a> {
    pub(crate) content: Text<'a>,
}

impl<'a> NestedListSubItemUi<'a> {
    pub fn new<T>(content: T) -> Self
    where
        T: Into<Text<'a>>,
    {
        Self {
            content: content.into(),
        }
    }
}

// #[derive(Clone)]
pub struct NestedListItemUi<'a> {
    pub(crate) content: Text<'a>,
    pub(crate) sub_items: Vec<NestedListSubItemUi<'a>>,
}

impl<'a> NestedListItemUi<'a> {
    pub fn new<T>(content: T) -> Self
    where
        T: Into<Text<'a>>,
    {
        Self {
            content: content.into(),
            sub_items: Vec::new(),
        }
    }

    pub fn sub_items(mut self, sub_items: Vec<NestedListSubItemUi<'a>>) -> Self {
        self.sub_items = sub_items;
        self
    }
}

// #[derive(Default)]
pub struct NestedListUi<'a> {
    pub(crate) block: Option<Block<'a>>,
    pub(crate) items: Vec<NestedListItemUi<'a>>,
    pub(crate) cursor: NestedCursor,
}

impl<'a> NestedListUi<'a> {
    pub fn new(items: Vec<NestedListItemUi<'a>>) -> Self {
        Self {
            block: None,
            items,
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

    pub fn with_items(mut self, items: Vec<NestedListItemUi<'a>>) -> Self {
        self.items = items;
        self
    }

    fn visible_list(&self, height: usize) -> (NestedCursor, NestedCursor) {
        if self.items.is_empty() {
            return (NestedCursor::empty(), NestedCursor::empty());
        }

        let mut slice_height = 0;
        let mut init_cursor = NestedCursor::from(0);
        let mut end_cursor = NestedCursor::empty();

        for item in &self.items {
            slice_height += 1; // sum the item

            let sub_items_count = item.sub_items.len();
            let missing_to_fill = height - slice_height;
            // println!("sub_items_count: {sub_items_count} - missing_to_fill: {missing_to_fill} , slice_height: {slice_height}");

            if sub_items_count >= missing_to_fill {
                end_cursor = NestedCursor::new(
                    end_cursor.idx().map(|c| c + 1).or(Some(0)),
                    if missing_to_fill > 0 {
                        Some(missing_to_fill.saturating_sub(1))
                    } else {
                        None
                    },
                );
                if init_cursor <= self.cursor && self.cursor <= end_cursor {
                    // in range
                    return (init_cursor, end_cursor);
                } else {
                    // end_cursor
                    let missing_children = sub_items_count - missing_to_fill;
                    init_cursor = if missing_children > 0 {
                        NestedCursor::new(
                            *end_cursor.idx(),
                            Some(missing_children.saturating_sub(1)),
                        )
                    } else {
                        NestedCursor::new(end_cursor.idx().map(|idx| idx + 1), None)
                    };
                    slice_height = 0
                }
            } else {
                // add and continue itering
                slice_height += sub_items_count;
                end_cursor = NestedCursor::new(
                    end_cursor.idx().map(|c| c + 1).or(Some(0)),
                    if item.sub_items.is_empty() {
                        None
                    } else {
                        Some(sub_items_count.saturating_sub(1))
                    },
                );
            }
        }

        (init_cursor, end_cursor)
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

        let range_cursor = self.visible_list(inner_area.height as usize);

        let init_cursor = range_cursor.0.inner();
        let end_cursor = range_cursor.1.inner();

        if init_cursor <= end_cursor {
            if let ((Some(init_idx), init_sub_idx), (Some(end_idx), end_sub_idx)) =
                (init_cursor, end_cursor)
            {
                // 1. Tomo un slice de los items
                // 2. comparo los  item indices
                // 3. Si el incice es del item inicial
                // 3.1 Verifico si el subindex es none -> renderizo solo el item, else -> tomo un slice de los hijos -> renderizo cada hijo

                let mut current_height = 0;
                for (i, item) in self
                    .items
                    .iter()
                    .enumerate()
                    .skip(init_idx)
                    .take((end_idx - init_idx).saturating_add(1))
                {
                    if i == init_idx {
                        match init_sub_idx {
                            Some(sub_idx) => {
                                for sub_item in item.sub_items.iter().skip(sub_idx) {
                                    let area = Rect {
                                        x: inner_area.top() + current_height,
                                        ..inner_area
                                    };
                                    sub_item.content.clone().render(area, buf);
                                    current_height += 1;
                                }
                            }
                            None => {
                                let area = Rect {
                                    x: inner_area.top() + current_height,
                                    ..inner_area
                                };
                                item.content.clone().render(area, buf);
                                current_height += 1;
                            }
                        }
                    } else if i == end_idx {
                        match end_sub_idx {
                            Some(sub_idx) => {
                                for sub_item in
                                    item.sub_items.iter().take(sub_idx.saturating_add(1))
                                {
                                    let area = Rect {
                                        x: inner_area.top() + current_height,
                                        ..inner_area
                                    };
                                    sub_item.content.clone().render(area, buf);
                                    current_height += 1;
                                }
                            }
                            None => {
                                let area = Rect {
                                    x: inner_area.top() + current_height,
                                    ..inner_area
                                };
                                item.content.clone().render(area, buf);
                                current_height += 1;
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_list_ui_empty_page_list() {
        let nested_list = NestedListUi::new(Vec::new());

        println!("{:?}", nested_list.visible_list(4));
    }

    #[test]
    fn nested_list_ui_page_list() {
        let empty_item = NestedListItemUi::new("empty_item");

        let item_with_children = NestedListItemUi::new("item-2_sub_4").sub_items(Vec::from([
            NestedListSubItemUi::new("item-2_sub_4__1"),
            NestedListSubItemUi::new("item-2_sub_4__2"),
            NestedListSubItemUi::new("item-2_sub_4__3"),
            NestedListSubItemUi::new("item-2_sub_4__4"),
        ]));

        let item_3_2 = NestedListItemUi::new("item-3_sub_2").sub_items(Vec::from([
            NestedListSubItemUi::new("item-3_sub_2__1"),
            NestedListSubItemUi::new("item-3_sub_2__2"),
            NestedListSubItemUi::new("item-3_sub_2__3"),
        ]));

        let nested_list = NestedListUi::new(Vec::from([
            empty_item,
            item_with_children,
            item_3_2,
            NestedListItemUi::new("item-4__empty"),
        ]))
        .with_cursor(NestedCursor::from((1, 2)));

        assert_eq!(
            nested_list.visible_list(4),
            (NestedCursor::from((1, 1)), NestedCursor::from((2, 2)))
        );

        println!("\n");

        let nested_list = nested_list.with_cursor(NestedCursor::from(0));

        assert_eq!(
            nested_list.visible_list(2),
            (NestedCursor::from(0), NestedCursor::from(1))
        );

        println!("\n");

        let nested_list = nested_list.with_cursor(NestedCursor::from(2));
        assert_eq!(
            nested_list.visible_list(6),
            (NestedCursor::from(2), NestedCursor::from(3))
        );
    }
}
