// use tui_tree_widget::Tree;

use std::collections::HashSet;

use ratatui::{
    layout::Rect,
    text::Span,
    widgets::{Block, Widget},
};

use super::state::Idx;

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

#[derive(Default)]
pub struct Collections<'a, 'b> {
    items: Vec<Item<'a>>,
    block: Option<Block<'b>>,
    openeds: HashSet<usize>,
    idx: Idx,
}

impl<'a, 'b> Collections<'a, 'b> {
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
            end = Idx::Parent(0);

            count_height += 1;

            if count_height >= height {
                if self.idx >= start && self.idx <= end {
                    break;
                } else {
                    start = end.clone();
                }
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

impl<'a, 'b> Widget for Collections<'a, 'b> {
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

        if let Some((start, end)) = self.get_page_items(area.height as usize) {
            (&self.items[start.0].label).render(acc_area, buf);
            acc_area.y += 1;

            if let Some(sub_i) = start.1 {
                for itm in self.items[start.0].children.get(sub_i..).unwrap() {
                    (&itm.label).render(acc_area, buf);
                    acc_area.y += 1;
                }
            }

            for i in (start.0 + 1)..(end.0 + 1) {
                (&self.items[i].label).render(acc_area, buf);
                acc_area.y += 1;

                if self.openeds.contains(&i) {
                    let end_list = end.1.unwrap_or(self.items[i].children.len());

                    for itm in &self.items[i].children[0..end_list] {
                        (&itm.label).render(acc_area, buf);
                        acc_area.y += 1;
                    }
                }
            }
        }
    }
}
