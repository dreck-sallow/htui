use crate::tools::tui::core::utils::{next_index, prev_index};

use super::cursor::NestedCursor;

pub struct NestedListItem<T, U> {
    inner: T,
    sub_items: Vec<U>,
}

impl<I, S> NestedListItem<I, S> {
    pub fn new(state: I, items: Vec<S>) -> Self {
        Self {
            inner: state,
            sub_items: items,
        }
    }
}

#[derive(Default)]
pub struct NestedListState<I, S> {
    cursor: NestedCursor,
    items: Vec<NestedListItem<I, S>>,
}

impl<I, S> NestedListState<I, S> {
    pub fn next(&mut self) {
        match self.cursor.inner() {
            (Some(item_idx), Some(sub_item_idx)) => {
                let item = &self.items[item_idx];

                match next_index(&item.sub_items, sub_item_idx) {
                    Some(next) => {
                        self.cursor = NestedCursor::from((item_idx, next));
                    }
                    None => {
                        if let Some(next) = next_index(&self.items, item_idx) {
                            self.cursor = NestedCursor::from(next);
                        }
                    }
                }
            }
            (Some(item_idx), None) => {
                let item = &self.items[item_idx];

                if !item.sub_items.is_empty() {
                    // self.cursor = (Some(item_idx), Some(0))
                    self.cursor = NestedCursor::from(item_idx);
                } else if let Some(next) = next_index(&self.items, item_idx) {
                    // self.cursor = (Some(next), None);
                    self.cursor = NestedCursor::from(next);
                }
            }
            _ => {
                if !self.items.is_empty() {
                    // self.cursor = (Some(0), None);
                    self.cursor = NestedCursor::from(0);
                }
            }
        }
    }

    pub fn prev(&mut self) {
        match self.cursor.inner() {
            (Some(item_idx), Some(sub_item_idx)) => {
                let item = &self.items[item_idx];

                match prev_index(&item.sub_items, sub_item_idx) {
                    Some(prev) => {
                        // self.cursor = (Some(item_idx), Some(prev));
                        self.cursor = NestedCursor::from((item_idx, prev));
                    }
                    None => {
                        if let Some(next) = prev_index(&self.items, item_idx) {
                            // self.cursor = (Some(next), None);
                            self.cursor = NestedCursor::from(next);
                        }
                    }
                }
            }
            (Some(item_idx), None) => {
                if item_idx == 0 {
                    return;
                }

                if let Some(prev_item) = &self.items.get(item_idx - 1) {
                    if !prev_item.sub_items.is_empty() {
                        // self.cursor =
                        self.cursor =
                            NestedCursor::from((item_idx - 1, prev_item.sub_items.len() - 1));
                        // self.cursor = (Some(item_idx - 1), Some(prev_item.sub_items.len() - 1))
                    } else if let Some(next) = prev_index(&self.items, item_idx) {
                        // self.cursor = (Some(next), None);
                        self.cursor = NestedCursor::from(next);
                    }
                }
            }
            _ => {
                if !self.items.is_empty() {
                    self.cursor = NestedCursor::from(0);
                    // self.cursor = (Some(0), None);
                }
            }
        }
    }

    pub fn append_item(&mut self, itm: NestedListItem<I, S>) {
        self.items.push(itm);
        if self.items.len() == 1 {
            self.cursor = NestedCursor::from(0);
            // self.cursor = (Some(0), None);
        }
    }

    pub fn append_sub_item(&mut self, itm: S) {
        if let (Some(idx), _) = self.cursor.inner() {
            self.items[idx].sub_items.push(itm);

            if self.items[idx].sub_items.len() == 1 {
                // self.cursor = (Some(idx), Some(0));
                self.cursor = NestedCursor::from((idx, 0));
            }
        }
    }

    pub fn get_current_item(&self) -> Option<&NestedListItem<I, S>> {
        match self.cursor.inner() {
            (Some(idx), _) => self.items.get(idx),
            _ => None,
        }
    }

    pub fn get_current_sub_item(&self) -> Option<&S> {
        if let (Some(idx), Some(sub_idx)) = self.cursor.inner() {
            return self
                .items
                .get(idx)
                .and_then(|itm| itm.sub_items.get(sub_idx));
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Dummy {
        name: &'static str,
    }

    impl Dummy {
        pub fn new(name: &'static str) -> Self {
            Self { name }
        }
    }

    #[test]
    fn test_empty_list() {
        let nested_list: NestedListState<(), ()> = NestedListState::default();
        assert_eq!(nested_list.cursor, NestedCursor::empty());
    }

    #[test]
    fn test_2item() {
        let mut nested_list: NestedListState<Dummy, ()> = NestedListState::default();

        nested_list.append_item(NestedListItem::new(Dummy::new("item_1"), [].into()));
        // assert_eq!(nested_list.cursor, (Some(0), None));
        assert_eq!(nested_list.cursor, NestedCursor::from(0));
    }

    #[test]
    fn test_next_items() {
        let mut nested_list: NestedListState<Dummy, ()> = NestedListState::default();

        nested_list.append_item(NestedListItem::new(Dummy::new("item_1"), [].into()));
        // assert_eq!(nested_list.cursor, (Some(0), None));
        assert_eq!(nested_list.cursor, NestedCursor::from(0));

        nested_list.append_item(NestedListItem::new(Dummy::new("item_1"), [].into()));
        // assert_eq!(nested_list.cursor, (Some(0), None));
        assert_eq!(nested_list.cursor, NestedCursor::from(0));

        nested_list.next();
        // assert_eq!(nested_list.cursor, (Some(1), None));
        assert_eq!(nested_list.cursor, NestedCursor::from(1));

        nested_list.next();
        // assert_eq!(nested_list.cursor, (Some(1), None));
        assert_eq!(nested_list.cursor, NestedCursor::from(1));
    }

    #[test]
    fn test_previous_items() {
        let mut nested_list: NestedListState<Dummy, ()> = NestedListState::default();
        nested_list.prev();
        // assert_eq!(nested_list.cursor, (None, None));
        assert_eq!(nested_list.cursor, NestedCursor::empty());

        nested_list.append_item(NestedListItem::new(Dummy::new("item_1"), [].into()));
        nested_list.next();

        nested_list.append_item(NestedListItem::new(Dummy::new("item_1"), [].into()));
        nested_list.next();

        // assert_eq!(nested_list.cursor, (Some(1), None));
        assert_eq!(nested_list.cursor, NestedCursor::from(1));

        nested_list.prev();
        // assert_eq!(nested_list.cursor, (Some(0), None));
        assert_eq!(nested_list.cursor, NestedCursor::from(0));

        nested_list.prev();
        // assert_eq!(nested_list.cursor, (Some(0), None));
        assert_eq!(nested_list.cursor, NestedCursor::from(0));

        nested_list.append_item(NestedListItem::new(Dummy::new("item_1"), [].into()));

        // assert_eq!(nested_list.cursor, (Some(0), None));
        assert_eq!(nested_list.cursor, NestedCursor::from(0));

        nested_list.next();
        // assert_eq!(nested_list.cursor, (Some(1), None));
        assert_eq!(nested_list.cursor, NestedCursor::from(1));

        nested_list.next();
        // assert_eq!(nested_list.cursor, (Some(2), None));
        assert_eq!(nested_list.cursor, NestedCursor::from(2));
    }
}
