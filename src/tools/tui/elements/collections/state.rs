use crate::tools::tui::core::elements::nested_list::{
    cursor::NestedCursor,
    state::{NestedListItem, NestedListState},
};

#[derive(Default)]
pub struct CollectionState {
    pub list: NestedListState<CollectionItem, RequestItem>,
}

impl CollectionState {
    pub fn add_item(&mut self, item: CollectionItem, sub_items: Vec<RequestItem>) {
        let nested_item = NestedListItem {
            inner: item,
            sub_items,
        };

        self.list.append_item(nested_item);
    }

    pub fn cursor(&self) -> NestedCursor {
        self.list.get_cursor().clone()
    }
}

#[derive(Default)]
pub struct CollectionItem {
    pub name: String,
}

impl CollectionItem {
    pub fn new<S>(name: S) -> Self
    where
        S: Into<String>,
    {
        Self { name: name.into() }
    }
}

#[derive(Default)]
pub struct RequestItem {
    pub name: String,
}

impl RequestItem {
    pub fn new<S>(name: S) -> Self
    where
        S: Into<String>,
    {
        Self { name: name.into() }
    }
}
