/// TODO: REFACTOR THE UI GENERATION
use ratatui::{
    layout::Rect,
    prelude::BlockExt,
    style::Style,
    text::Text,
    widgets::{Block, Widget},
};

use super::cursor::NestedCursor;

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

    /// method used to compute the init cursor and end cursor
    /// for the slice height
    fn visible_list(&self, height: usize) -> (NestedCursor, NestedCursor) {
        if self.items.is_empty() || self.cursor == NestedCursor::empty() {
            return (NestedCursor::empty(), NestedCursor::empty());
        }

        #[derive(Debug)]
        struct Itm {
            height: usize,
            is_sub_item: bool,
        }

        let mut current_cursor = NestedCursor::from(0);

        let mut iter = || {
            if let (Some(idx), sub_idx_opt) = current_cursor.inner() {
                if let Some(item) = self.items.get(idx) {
                    match sub_idx_opt {
                        None => {
                            // only parent item
                            current_cursor.set_sub_idx(Some(0));

                            return Some(Itm {
                                height: item.content.height(),
                                is_sub_item: false,
                            });
                        }
                        Some(sub_idx) => match item.sub_items.get(sub_idx) {
                            Some(sub_itm) => {
                                current_cursor.set_sub_idx(Some(sub_idx + 1));

                                return Some(Itm {
                                    height: sub_itm.content.height(),
                                    is_sub_item: true,
                                });
                            }
                            None => {
                                if let Some(next_itm) = self.items.get(idx + 1) {
                                    current_cursor.set_idx(Some(idx + 1));
                                    current_cursor.set_sub_idx(Some(0));

                                    return Some(Itm {
                                        height: next_itm.content.height(),
                                        is_sub_item: false,
                                    });
                                }
                            }
                        },
                    }
                }
            }
            None
        };

        // compute visible list
        let mut current_height = 0;
        let mut init_cursor = NestedCursor::from(0);
        let mut end_cursor = NestedCursor::empty();

        while let Some(nested_itm) = iter() {
            if current_height + nested_itm.height <= height {
                // continue iterating
                current_height += nested_itm.height;

                if nested_itm.is_sub_item {
                    end_cursor.set_sub_idx(Some(end_cursor.sub_idx().map_or(0, |i| i + 1)));
                } else {
                    end_cursor.set_idx(Some(end_cursor.idx().map_or(0, |i| i + 1)));
                    end_cursor.set_sub_idx(None);
                }
            } else {
                // end of the page
                if init_cursor <= self.cursor && self.cursor <= end_cursor {
                    return (init_cursor, end_cursor);
                } else {
                    if nested_itm.is_sub_item {
                        end_cursor.set_sub_idx(Some(end_cursor.sub_idx().map_or(0, |i| i + 1)));
                    } else {
                        end_cursor.set_idx(Some(end_cursor.idx().map_or(0, |i| i + 1)));

                        end_cursor.set_sub_idx(None);
                    }

                    current_height = nested_itm.height;
                    init_cursor = end_cursor.clone();
                }
            }
        }

        (init_cursor, end_cursor)
    }
}

impl Widget for NestedListUi<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        self.block.render(area, buf);
        let inner_area = self.block.inner_if_some(area);

        if inner_area.is_empty() || self.items.is_empty() {
            return;
        }

        let (init_cursor, end_cursor) = self.visible_list(inner_area.height as usize);

        if init_cursor <= end_cursor {
            if let ((Some(init_idx), init_sub_idx_opt), (Some(end_idx), end_sub_idx_opt)) =
                (init_cursor.inner(), end_cursor.inner())
            {
                let mut current_height: usize = 0;

                for (i, item) in self
                    .items
                    .iter()
                    .enumerate()
                    .skip(init_idx)
                    .take((end_idx - init_idx).saturating_add(1))
                {
                    let sub_items: Vec<(usize, &NestedListSubItemUi<'_>)> = if init_idx == i {
                        item.sub_items
                            .iter()
                            .enumerate()
                            .skip(init_sub_idx_opt.unwrap_or(0))
                            .collect()
                    } else if end_idx == i {
                        item.sub_items
                            .iter()
                            .enumerate()
                            .take(end_sub_idx_opt.map_or(item.sub_items.len(), |i| i + 1))
                            .collect()
                    } else {
                        item.sub_items.iter().enumerate().collect()
                    };

                    // Render item and sub_item

                    let area = Rect {
                        y: inner_area.top() + current_height as u16,
                        height: item.content.height() as u16,
                        ..inner_area
                    };

                    let is_selected_itm = self.cursor.idx().is_some_and(|idx| idx == i)
                        & self.cursor.sub_idx().is_none();

                    let content = if is_selected_itm {
                        item.content
                            .clone()
                            .patch_style(Style::default().bg(ratatui::style::Color::Red))
                    } else {
                        item.content.clone()
                    };

                    content.render(area, buf);
                    current_height += item.content.height();

                    if is_selected_itm {
                        buf.set_stringn(
                            inner_area.x,
                            current_height as u16,
                            "> ",
                            inner_area.width as usize,
                            Style::default(),
                        );
                    }

                    for (sub_i, sub_item) in sub_items {
                        let is_sub_item_selected = self
                            .cursor
                            .sub_idx()
                            .is_some_and(|sub_idx| sub_idx == sub_i)
                            && self.cursor.idx().is_some_and(|idx| idx == i);

                        let content = if is_sub_item_selected {
                            sub_item
                                .content
                                .clone()
                                .patch_style(Style::default().bg(ratatui::style::Color::Yellow))
                        } else {
                            sub_item.content.clone()
                        };

                        let area = Rect {
                            y: inner_area.top() + current_height as u16,
                            height: sub_item.content.height() as u16,
                            ..inner_area
                        };

                        content.render(area, buf);
                        current_height += sub_item.content.height();

                        buf.set_stringn(
                            inner_area.x,
                            current_height as u16,
                            if is_sub_item_selected { "  > " } else { "  " },
                            inner_area.width as usize,
                            Style::default(),
                        );
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
            (NestedCursor::from((1, 2)), NestedCursor::from((2, 0)))
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
