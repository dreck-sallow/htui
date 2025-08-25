use std::collections::HashSet;

use ratatui::{
    layout::Rect,
    style::Style,
    symbols,
    text::Span,
    widgets::{Block, Widget},
};

use super::state::Idx;

#[derive(Debug)]
pub struct Item<'a> {
    label: Span<'a>,
    children: Vec<Item<'a>>,
}

impl<'a> Item<'a> {
    pub fn new<Label: Into<Span<'a>>>(label: Label) -> Self {
        Self {
            label: label.into(),
            children: Vec::new(),
        }
    }

    pub fn add_child(&mut self, item: Item<'a>) {
        self.children.push(item);
    }
}

pub struct CollectionList<'a, 'b> {
    items: Vec<Item<'a>>,
    block: Option<Block<'b>>,
    highlight_style: Style,
    open_symbol: &'static str,
    close_symbol: &'static str,
    openeds: HashSet<usize>,
    idx: Idx,
}

impl Default for CollectionList<'_, '_> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            block: None,
            highlight_style: Style::default(),
            open_symbol: "\u{25bc} ",
            close_symbol: "\u{25b6} ",
            openeds: HashSet::default(),
            idx: Idx::None,
        }
    }
}

impl<'a, 'b> CollectionList<'a, 'b> {
    pub fn set_items(mut self, items: Vec<Item<'a>>) -> Self {
        self.items = items;
        self
    }

    pub fn set_block(mut self, block: Block<'b>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn set_openeds(mut self, openeds: HashSet<usize>) -> Self {
        self.openeds = openeds;
        self
    }

    pub fn set_idx(mut self, idx: Idx) -> Self {
        self.idx = idx;
        self
    }

    pub fn set_highlight_style(mut self, style: Style) -> Self {
        self.highlight_style = style;
        self
    }

    fn get_page_items(
        &self,
        height: usize,
    ) -> Option<((usize, Option<usize>), (usize, Option<usize>))> {
        if self.items.is_empty() {
            return None;
        }

        let mut start = Idx::Parent(0);
        let mut end = start;

        let mut count_height = 0;

        for (i, item) in self.items.iter().enumerate() {
            end = Idx::Parent(i);

            count_height += 1;

            if count_height >= height {
                if self.idx >= start && self.idx <= end {
                    break;
                } else {
                    start = end;
                }
                count_height = 0;
            }

            // NOTE: Only walk over children when the parent is opened
            if self.openeds.contains(&i) {
                for (sub_i, _sub_item) in item.children.iter().enumerate() {
                    end = Idx::Child(i, sub_i);

                    count_height += 1;

                    if count_height >= height {
                        if self.idx >= start && self.idx <= end {
                            break;
                        } else {
                            start = end;
                        }
                    }
                }
            }
        }

        let idx_to_raw = |idx: Idx| match idx {
            Idx::None => unreachable!(),
            Idx::Parent(i) => (i, None),
            Idx::Child(i, sub_i) => (i, Some(sub_i)),
        };

        Some((idx_to_raw(start), idx_to_raw(end)))
    }
}

impl Widget for CollectionList<'_, '_> {
    fn render(self, mut area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        if area.is_empty() {
            return;
        }

        // Draw the block, and set the area from the block inner_area
        if let Some(block) = &self.block {
            let inner_area = block.inner(area);
            block.render(area, buf);
            area = inner_area
        };

        // Draw the items
        // let mut top = area.top();
        if let Some((start, end)) = self.get_page_items(area.height as usize) {
            let mut line_area = Rect { height: 1, ..area };

            // Walk over the items page slice
            for (collection_loop_i, collection) in
                self.items[start.0..(end.0 + 1)].iter().enumerate()
            {
                let (show_collection, children, padded_children): (bool, &[Item<'_>], usize) = {
                    if (start.0 + collection_loop_i) == start.0 && start.0 == end.0 {
                        // We have only 1 collection, then check the children
                        match (start.1, end.1) {
                            (None, None) => (true, &[], 0),
                            (None, Some(end_children)) => (true, &self.items[0..end_children], 0),
                            (Some(start_children), Some(end_children)) => (
                                false,
                                &self.items[start_children..end_children],
                                start_children,
                            ),
                            (Some(_), None) => {
                                // This cannot would happened
                                unreachable!()
                            }
                        }
                    } else if (start.0 + collection_loop_i) == start.0 {
                        let children_start_idx = start.1.unwrap_or(0);
                        (
                            true,
                            &self.items[start.0].children[children_start_idx..],
                            children_start_idx,
                        )
                    } else if (start.0 + collection_loop_i) == end.0 {
                        (
                            true,
                            &self.items[end.0].children
                                [0..end.1.unwrap_or(self.items[end.0].children.len())],
                            0,
                        )
                    } else {
                        (true, &self.items[collection_loop_i].children, 0)
                    }
                };

                if show_collection {
                    // Draw the collection
                    let symbol = if self.openeds.contains(&(start.0 + collection_loop_i)) {
                        self.open_symbol
                    } else {
                        self.close_symbol
                    };

                    // Draw the open/close collection symbol
                    let (x, y) = buf.set_stringn(
                        line_area.x,
                        line_area.y,
                        symbol,
                        symbol.len(),
                        self.items[start.0].label.style,
                    );

                    // Draw the collection label, and add 1 to top area
                    buf.set_span(x, y, &collection.label, collection.label.width() as u16);
                    match self.idx {
                        Idx::Parent(selected_idx) => {
                            if selected_idx == (collection_loop_i + start.0) {
                                buf.set_style(line_area, self.highlight_style);
                            }
                        }
                        _ => {}
                    }
                    line_area.y += 1;
                }

                if self.openeds.contains(&(start.0 + collection_loop_i)) {
                    for (i, child) in children.iter().enumerate() {
                        let border_tree_symbol = if i == children.len() - 1 {
                            symbols::line::BOTTOM_LEFT
                        } else {
                            symbols::line::VERTICAL_RIGHT
                        };

                        let (mut x, y) = buf.set_stringn(
                            line_area.x + 1,
                            line_area.y,
                            border_tree_symbol,
                            border_tree_symbol.len(),
                            child.label.style,
                        );
                        x += 1;

                        buf.set_span(x, y, &child.label, child.label.width() as u16);

                        match self.idx {
                            Idx::Child(collection_idx, request_idx) => {
                                if collection_idx == (collection_loop_i + start.0)
                                    && (padded_children + i) == request_idx
                                {
                                    buf.set_style(
                                        // Rect {
                                        //     x,
                                        //     y,
                                        //     width: child.label.width() as u16,
                                        //     height: 1,
                                        // },
                                        line_area,
                                        self.highlight_style,
                                    );
                                }
                            }
                            _ => {}
                        }

                        line_area.y += 1;
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
    fn test_opened() {
        let mut parent_a = Item::new("parent_a");
        parent_a.add_child(Item::new("child a_a"));
        parent_a.add_child(Item::new("child a_b"));
        // parent_a.add_child(Item::new("child a_c"));

        let mut parent_b = Item::new("parent_b");
        parent_b.add_child(Item::new("child b_a"));
        // parent_b.add_child(Item::new("child b_b"));
        // parent_b.add_child(Item::new("child b_c"));
        // parent_b.add_child(Item::new("child b_d"));

        let collections = CollectionList::default()
            .set_items(vec![parent_a, parent_b])
            .set_openeds(HashSet::from([0, 1]))
            .set_idx(Idx::Child(0, 0));

        assert_eq!(
            collections.get_page_items(49),
            Some(((0, None), (1, Some(0))))
        );
    }
}
