use crate::tools::tui::core::utils::{next_index, prev_index};

use super::{
    cursor::NestedCursor,
    item_v2::{NestedListItem, NestedListItemState},
    Idx,
};

#[derive(Default)]
pub struct NestedListStateV2<S, M> {
    cursor: NestedCursor,
    list: Vec<NestedListItem<S, M>>,
}

impl<S, M> NestedListStateV2<S, M> {
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

    pub fn current_inner(&self) -> Option<NestedListItemState<&S, &M>> {
        self.cursor.idx().map(|idx| match &self.list[idx] {
            NestedListItem::Single(single) => NestedListItemState::Single(single),
            NestedListItem::Group { inner, items } => match self.cursor.sub_idx() {
                Some(sub_idx) => {
                    if let NestedListItem::Single(single) = &items[*sub_idx] {
                        NestedListItemState::Single(single)
                    } else {
                        unreachable!()
                    }
                }
                None => NestedListItemState::Group(inner),
            },
        })
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
                match &mut self.list[*idx] {
                    NestedListItem::Single(_) => {
                        self.list.remove(*idx);
                        if *idx <= self.cursor.idx().unwrap() {
                            self.cursor.reduce_idx(1);
                        }
                    }
                    NestedListItem::Group { items, .. } => match self.cursor.sub_idx() {
                        Some(sub_idx) => {
                            items.remove(*sub_idx);

                            if *sub_idx <= self.cursor.sub_idx().unwrap() {
                                self.cursor.reduce_sub_idx(1);
                            }
                        }
                        None => {
                            self.list.remove(*idx);
                            if *idx <= self.cursor.idx().unwrap() {
                                self.cursor.reduce_idx(1);
                            }
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

    pub fn next_v2<F>(&mut self, checker: F)
    where
        F: Fn(&NestedListItem<S, M>) -> bool,
    {
        if let Some(idx) = self.cursor.idx() {
            let item = &self.list[*idx];
            let is_skipped = !checker(item);

            #[inline]
            fn calc_idx<T>(vec: &Vec<T>, i: usize, is_skipped: bool) -> Option<usize> {
                if is_skipped {
                    next_index(vec, i).and_then(|next_idx| next_index(vec, next_idx))
                } else {
                    next_index(vec, i)
                }
            }

            match item {
                NestedListItem::Single(_) => {
                    if let Some(next_idx) = calc_idx(&self.list, *idx, is_skipped) {
                        self.cursor.set_idx(Some(next_idx));
                    }
                }
                NestedListItem::Group { items, .. } => {
                    if is_skipped {
                        if let Some(next_idx) = calc_idx(&self.list, *idx, is_skipped) {
                            self.cursor.set_idx(Some(next_idx));
                        }
                    } else {
                        let sub_idx_opt = self
                            .cursor
                            .sub_idx()
                            .and_then(|sub_idx| calc_idx(items, sub_idx, checker(&items[sub_idx])));

                        match sub_idx_opt {
                            Some(_) => todo!(),
                            None => {
                                if !items.is_empty() {
                                    self.cursor.add_sub_idx(1);
                                } else if let Some(next_idx) = next_index(&self.list, *idx) {
                                    self.cursor.set_idx(Some(next_idx));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn prev_v2<F>(&mut self, checker: F)
    where
        F: Fn(&NestedListItem<S, M>) -> bool,
    {
        if let Some(idx) = self.cursor.idx() {
            let item = &self.list[*idx];
            let is_skipped = !checker(item);

            #[inline]
            fn calc_idx(i: usize, is_skipped: bool) -> Option<usize> {
                if is_skipped {
                    prev_index(i).and_then(prev_index)
                } else {
                    prev_index(i)
                }
            }

            match item {
                NestedListItem::Single(_) => {
                    if let Some(prev_idx) = calc_idx(*idx, is_skipped) {
                        self.cursor.set_idx(Some(prev_idx));
                    }
                }
                NestedListItem::Group { items, .. } => match self.cursor.sub_idx() {
                    Some(sub_idx) => {
                        if let Some(prev_idx) = calc_idx(*sub_idx, checker(&items[*sub_idx])) {
                            self.cursor.set_sub_idx(Some(prev_idx));
                        }
                    }
                    None => {
                        if let Some(prev_idx) = calc_idx(*idx, is_skipped) {
                            self.cursor.set_idx(Some(prev_idx));
                        }
                    }
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn create_state() -> NestedListStateV2<String, String> {
        NestedListStateV2::default()
    }

    fn next_count<S, G>(state: &mut NestedListStateV2<S, G>, count: u8) {
        for _i in 0..count {
            state.next_v2(|_| true);
        }
    }

    fn prev_count<S, G>(state: &mut NestedListStateV2<S, G>, count: u8) {
        for _i in 0..count {
            state.prev_v2(|_| true);
        }
    }

    fn create_single(str: &str) -> NestedListItem<String, String> {
        NestedListItem::Single(str.into())
    }

    fn create_multiple(str: &str, items: Vec<&'static str>) -> NestedListItem<String, String> {
        let children = {
            let mut list = Vec::new();

            for item in items {
                list.push(create_single(item));
            }

            list
        };

        NestedListItem::Group {
            inner: String::from(str),
            items: children,
        }
    }

    #[test]
    fn test_empty_v2() {
        let mut state = create_state();

        next_count(&mut state, 3);

        assert_eq!(state.cursor, NestedCursor::empty());
    }

    #[test]
    fn test_walk_over_list() {
        let mut state = create_state();

        prev_count(&mut state, 3);
        assert_eq!(state.cursor, NestedCursor::empty());

        state.insert(create_single("single 1"));
        next_count(&mut state, 1);
        // state.next();
        assert_eq!(state.cursor, NestedCursor::from(0));

        state.insert(create_single("single 2"));
        next_count(&mut state, 1);
        // state.next();
        assert_eq!(state.cursor, NestedCursor::from(1));

        // prev
        prev_count(&mut state, 2);
        assert_eq!(state.cursor, NestedCursor::from(0));

        // next 2
        next_count(&mut state, 2);
        // state.next();
        // state.next();

        state.insert(create_multiple(
            "multiple 1",
            ["sub 1", "sub 2", "sub 3"].into(),
        ));

        next_count(&mut state, 1);
        // state.next();
        assert_eq!(state.cursor, NestedCursor::from(2));

        next_count(&mut state, 2);
        // state.next();
        // state.next();
        assert_eq!(state.cursor, NestedCursor::from((2, 1)));

        next_count(&mut state, 4);
        // state.next();
        // state.next();
        // state.next();
        // state.next();
        assert_eq!(state.cursor, NestedCursor::from((2, 2)));

        state.insert(create_single("single 3"));
        next_count(&mut state, 1);
        // state.next();
        assert_eq!(state.cursor, NestedCursor::from(3));

        prev_count(&mut state, 1);
        // state.prev();
        assert_eq!(state.cursor, NestedCursor::from((2, 2)));

        prev_count(&mut state, 1);
        // state.prev();
        assert_eq!(state.cursor, NestedCursor::from((2, 1)));

        prev_count(&mut state, 1);
        // state.prev();
        assert_eq!(state.cursor, NestedCursor::from((2, 0)));

        prev_count(&mut state, 1);
        // state.prev();
        assert_eq!(state.cursor, NestedCursor::from(2));

        prev_count(&mut state, 1);
        // state.prev();
        assert_eq!(state.cursor, NestedCursor::from(1));
    }
}
