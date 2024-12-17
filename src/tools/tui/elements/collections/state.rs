use crate::{tools::tui::core::elements::nested_list::NestedListState, tui::core::elements};

#[derive(Default)]
pub struct CollectionState {
    pub list: NestedListState<CollectionItem, RequestItem>,
}

#[derive(Default)]
pub struct CollectionItem {
    name: String,
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
    name: String,
}

impl RequestItem {
    pub fn new<S>(name: S) -> Self
    where
        S: Into<String>,
    {
        Self { name: name.into() }
    }
}
