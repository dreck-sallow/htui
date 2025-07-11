use std::collections::HashSet;

use ratatui::{
    layout::Rect,
    style::Style,
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

impl<'a, 'b> Default for CollectionList<'a, 'b> {
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
        let mut end = start.clone();

        let mut count_height = 0;

        for (i, item) in self.items.iter().enumerate() {
            end = Idx::Parent(i);

            count_height += 1;

            if count_height >= height {
                if self.idx >= start && self.idx <= end {
                    break;
                } else {
                    start = end.clone();
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
                            start = end.clone();
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

impl<'a, 'b> Widget for CollectionList<'a, 'b> {
    fn render(self, mut area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        if area.is_empty() {
            return;
        }

        if let Some(block) = &self.block {
            let inner_area = block.inner(area);

            block.render(area, buf);

            area = inner_area
        };

        let mut acc_area = Rect { height: 1, ..area };

        fn is_selected(idx: Idx, cursor: (usize, Option<usize>)) -> bool {
            match idx {
                Idx::None => false,
                Idx::Parent(i) => (cursor.0 == i) && cursor.1.is_none(),
                Idx::Child(i, sub_i) => {
                    (cursor.0 == i)
                        && cursor
                            .1
                            .map(|cursor_sub_i| cursor_sub_i == sub_i)
                            .unwrap_or_default()
                }
            }
        }

        let blank_symbol = "  ";

        if let Some((start, end)) = self.get_page_items(area.height as usize) {
            let symbol = if self.openeds.contains(&start.0) {
                self.open_symbol
            } else {
                self.close_symbol
            };
            let (x, _) = buf.set_stringn(
                acc_area.x,
                acc_area.y,
                symbol,
                symbol.len(),
                self.items[start.0].label.style,
            );
            let mut render_area = acc_area.clone();
            render_area.x = x;

            (&self.items[start.0].label).render(render_area, buf);
            if is_selected(self.idx, start) {
                buf.set_style(acc_area, self.highlight_style);
            }
            acc_area.y += 1;

            if self.openeds.contains(&start.0) {
                for (sub_i, itm) in self.items[start.0].children[start.1.unwrap_or(0)..]
                    .iter()
                    .enumerate()
                {
                    let (x, _) = buf.set_stringn(
                        acc_area.x,
                        acc_area.y,
                        blank_symbol,
                        blank_symbol.len(),
                        itm.label.style,
                    );
                    let mut render_area = acc_area.clone();
                    render_area.x = x;

                    (&itm.label).render(render_area, buf);

                    if is_selected(self.idx, (start.0, Some(sub_i))) {
                        buf.set_style(acc_area, self.highlight_style);
                    }
                    acc_area.y += 1;
                }
            }

            for i in (start.0 + 1)..(end.0 + 1) {
                let symbol = if self.openeds.contains(&i) {
                    self.open_symbol
                } else {
                    self.close_symbol
                };

                let (x, _) = buf.set_stringn(
                    acc_area.x,
                    acc_area.y,
                    symbol,
                    symbol.len(),
                    self.items[i].label.style,
                );
                let mut render_area = acc_area.clone();
                render_area.x = x;

                (&self.items[i].label).render(render_area, buf);

                if is_selected(self.idx, (i, None)) {
                    buf.set_style(acc_area, self.highlight_style);
                }

                acc_area.y += 1;

                let current_len = self.items[i].children.len();
                if self.openeds.contains(&i) && current_len > 0 {
                    let end_list = end.1.unwrap_or(current_len - 1);

                    for (sub_i, itm) in self.items[i].children[0..(end_list + 1)].iter().enumerate()
                    {
                        let (x, _) = buf.set_stringn(
                            acc_area.x,
                            acc_area.y,
                            blank_symbol,
                            blank_symbol.len(),
                            itm.label.style,
                        );
                        let mut render_area = acc_area.clone();
                        render_area.x = x;

                        (&itm.label).render(render_area, buf);
                        if is_selected(self.idx, (i, Some(sub_i))) {
                            buf.set_style(acc_area, self.highlight_style);
                        }
                        acc_area.y += 1;
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
