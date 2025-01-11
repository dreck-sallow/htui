use crate::tools::tui::core::utils::next_index;

use super::{
    cursor::NestedCursor,
    item::{NestedListMultiple, NestedListSingle},
    Idx,
};

/// Type used for normalize the two diferent branches
/// for each item
pub enum NestedListItem<S, M> {
    Sigle(NestedListSingle<S>),
    Multiple(NestedListMultiple<M, S>),
}

#[derive(Default)]
pub struct NestedListStateV2<S, M> {
    cursor: NestedCursor,
    list: Vec<NestedListItem<S, M>>,
}

impl<S, M> NestedListStateV2<S, M> {
    pub fn new() -> Self {
        Self {
            cursor: NestedCursor::empty(),
            list: Vec::new(),
        }
    }

    pub fn items(&self) -> &Vec<NestedListItem<S, M>> {
        &self.list
    }

    /// Get the current cursor copy
    pub fn cursor(&self) -> NestedCursor {
        self.cursor.clone()
    }

    /// Get the inner current cursor indices
    pub fn index(&self) -> (Idx, Idx) {
        self.cursor.inner()
    }

    pub fn insert(&mut self, item: NestedListItem<S, M>) {
        self.list.push(item);

        if self.cursor == NestedCursor::empty() {
            self.cursor.add_idx(0);
        }
    }

    pub fn remove_by_cursor(&mut self, cursor: NestedCursor) {
        if let Some(idx) = cursor.idx() {
            if *idx <= (self.list.len().saturating_sub(1)) {
                match self.list[*idx] {
                    NestedListItem::Sigle(_) => {
                        self.list.remove(*idx);

                        if *idx >= self.cursor.idx().unwrap() {
                            self.cursor.reduce_idx(1);
                        }
                    }
                    NestedListItem::Multiple(ref mut multiple) => match self.cursor.sub_idx() {
                        None => {
                            self.list.remove(*idx);

                            if *idx >= self.cursor.idx().unwrap() {
                                self.cursor.reduce_idx(1);
                            }
                        }
                        Some(sub_idx) => {
                            multiple.remove_child(*sub_idx);
                            self.cursor.reduce_sub_idx(1)
                        }
                    },
                }
            }
        }
    }

    /// Remove the item reading the self current cursor
    pub fn remove(&mut self) {
        self.remove_by_cursor(self.cursor.clone());
    }

    pub fn next(&mut self) {
        if let (Some(idx), sub_idx_opt) = self.index() {
            match self.list[idx] {
                NestedListItem::Sigle(_) => {
                    if let Some(next_idx) = next_index(&self.list, idx) {
                        self.cursor.set_idx(Some(next_idx));
                    }
                }
                NestedListItem::Multiple(ref multiple) => {
                    let has_no_next_index = match sub_idx_opt {
                        Some(sub_idx) => sub_idx >= multiple.count_children().saturating_sub(1),
                        None => !multiple.has_children(),
                    };

                    if has_no_next_index {
                        if let Some(next_idx) = next_index(&self.list, idx) {
                            self.cursor.set_idx(Some(next_idx));
                        }
                    } else {
                        self.cursor.add_sub_idx(1);
                    }
                }
            }
        }
    }

    pub fn prev(&mut self) {
        if let (Some(idx), sub_idx_opt) = self.index() {
            let is_prev_idx = match self.list[idx] {
                NestedListItem::Sigle(_) => true,
                NestedListItem::Multiple(_) => sub_idx_opt.is_none(),
            };

            if is_prev_idx {
                if idx > 0 {
                    match &self.list[idx.saturating_sub(1)] {
                        NestedListItem::Sigle(_) => {
                            self.cursor.reduce_idx(1);
                        }
                        NestedListItem::Multiple(ref multiple) => {
                            if multiple.has_children() {
                                self.cursor = NestedCursor::from((
                                    idx.saturating_sub(1),
                                    multiple.count_children().saturating_sub(1),
                                ));
                            } else {
                                self.cursor.reduce_idx(1);
                            }
                        }
                    }
                }
            } else {
                self.cursor.reduce_sub_idx(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn create_state() -> NestedListStateV2<String, String> {
        NestedListStateV2::new()
    }

    fn create_single(str: &str) -> NestedListItem<String, String> {
        NestedListItem::Sigle(NestedListSingle(String::from(str)))
    }

    fn create_multiple(str: &str, items: Vec<&'static str>) -> NestedListItem<String, String> {
        let children = {
            let mut list = Vec::new();

            for item in items {
                list.push(NestedListSingle(item.to_string()));
            }

            list
        };

        let multiple = NestedListMultiple::new(String::from(str)).with_children(children);
        NestedListItem::Multiple(multiple)
    }

    #[test]
    fn test_empty_v2() {
        let mut state = create_state();
        state.next();
        state.next();
        state.next();

        assert_eq!(state.cursor, NestedCursor::empty());
    }

    #[test]
    fn test_walk_over_list() {
        let mut state = create_state();

        state.prev();
        state.prev();
        state.prev();
        assert_eq!(state.cursor, NestedCursor::empty());

        state.insert(create_single("single 1"));
        state.next();
        assert_eq!(state.cursor, NestedCursor::from(0));

        state.insert(create_single("single 2"));
        state.next();
        assert_eq!(state.cursor, NestedCursor::from(1));

        // prev
        state.prev();
        state.prev();
        assert_eq!(state.cursor, NestedCursor::from(0));

        // next 2
        state.next();
        state.next();

        state.insert(create_multiple(
            "multiple 1",
            ["sub 1", "sub 2", "sub 3"].into(),
        ));

        state.next();
        assert_eq!(state.cursor, NestedCursor::from(2));

        state.next();
        state.next();
        assert_eq!(state.cursor, NestedCursor::from((2, 1)));

        state.next();
        state.next();
        state.next();
        state.next();
        assert_eq!(state.cursor, NestedCursor::from((2, 2)));

        state.insert(create_single("single 3"));
        state.next();
        assert_eq!(state.cursor, NestedCursor::from(3));

        state.prev();
        assert_eq!(state.cursor, NestedCursor::from((2, 2)));

        state.prev();
        assert_eq!(state.cursor, NestedCursor::from((2, 1)));

        state.prev();
        assert_eq!(state.cursor, NestedCursor::from((2, 0)));

        state.prev();
        assert_eq!(state.cursor, NestedCursor::from(2));

        state.prev();
        assert_eq!(state.cursor, NestedCursor::from(1));
    }
}
