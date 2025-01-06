use crate::tools::tui::core::utils::{next_index, prev_index};

use super::cursor::NestedCursor;

pub struct NestedListItem<T, U> {
    pub inner: T,
    pub sub_items: Vec<U>,
}

impl<I, S> NestedListItem<I, S> {
    pub fn new(state: I) -> Self {
        Self {
            inner: state,
            sub_items: Vec::new(),
        }
    }
}

/// State for ´nestedList´ widget, that handle the
/// items(subitems) and the cursor state
#[derive(Default)]
pub struct NestedListState<I, S> {
    cursor: NestedCursor,
    items: Vec<NestedListItem<I, S>>,
}

impl<I, S> NestedListState<I, S> {
    pub fn items_ref(&self) -> &Vec<NestedListItem<I, S>> {
        &self.items
    }
    pub fn get_cursor(&self) -> &NestedCursor {
        &self.cursor
    }

    pub fn next(&mut self) {
        if let (Some(idx), sub_idx_opt) = self.cursor.inner() {
            let item = &self.items[idx];

            let sub_index = match sub_idx_opt {
                Some(sub_idx) => next_index(&item.sub_items, sub_idx),
                None => (!item.sub_items.is_empty()).then_some(0),
            };

            match sub_index {
                Some(next_sub_idx) => self.cursor = NestedCursor::from((idx, next_sub_idx)),
                None => {
                    if let Some(next) = next_index(&self.items, idx) {
                        self.cursor = NestedCursor::from(next);
                    }
                }
            }
        }
    }

    pub fn prev(&mut self) {
        if let (Some(idx), sub_idx_opt) = self.cursor.inner() {
            let item = &self.items[idx];

            match sub_idx_opt {
                Some(sub_idx) => match prev_index(&item.sub_items, sub_idx) {
                    Some(prev_idx) => self.cursor.set_sub_idx(Some(prev_idx)),
                    None => self.cursor = NestedCursor::from(idx),
                },
                None => {
                    if let Some(prev_idx) = prev_index(&self.items, idx) {
                        let prev_item = &self.items[prev_idx];

                        if !prev_item.sub_items.is_empty() {
                            self.cursor =
                                NestedCursor::from((prev_idx, prev_item.sub_items.len() - 1));
                        } else {
                            self.cursor = NestedCursor::from(prev_idx);
                        }
                    }
                }
            }
        }
    }

    pub fn append_item(&mut self, itm: NestedListItem<I, S>) {
        self.items.push(itm);
        if self.items.len() == 1 {
            self.cursor = NestedCursor::from(0);
        }
    }

    pub fn append_sub_item(&mut self, itm: S) {
        if let (Some(idx), _) = self.cursor.inner() {
            self.items[idx].sub_items.push(itm);

            if self.items[idx].sub_items.len() == 1 {
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

        nested_list.append_item(NestedListItem::new(Dummy::new("item_1")));
        // assert_eq!(nested_list.cursor, (Some(0), None));
        assert_eq!(nested_list.cursor, NestedCursor::from(0));
    }

    #[test]
    fn test_next_items() {
        let mut nested_list: NestedListState<Dummy, ()> = NestedListState::default();

        nested_list.append_item(NestedListItem::new(Dummy::new("item_1")));
        // assert_eq!(nested_list.cursor, (Some(0), None));
        assert_eq!(nested_list.cursor, NestedCursor::from(0));

        nested_list.append_item(NestedListItem::new(Dummy::new("item_1")));
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
        assert_eq!(nested_list.cursor, NestedCursor::empty());

        nested_list.append_item(NestedListItem::new(Dummy::new("item_1")));
        nested_list.next();

        assert_eq!(nested_list.cursor, NestedCursor::from(0));

        nested_list.append_item(NestedListItem::new(Dummy::new("item_1")));
        nested_list.next();

        assert_eq!(nested_list.cursor, NestedCursor::from(1));

        nested_list.prev();
        assert_eq!(nested_list.cursor, NestedCursor::from(0));

        nested_list.prev();
        assert_eq!(nested_list.cursor, NestedCursor::from(0));

        nested_list.append_item(NestedListItem::new(Dummy::new("item_1")));

        assert_eq!(nested_list.cursor, NestedCursor::from(0));

        nested_list.next();
        assert_eq!(nested_list.cursor, NestedCursor::from(1));

        nested_list.next();
        assert_eq!(nested_list.cursor, NestedCursor::from(2));
    }
}
