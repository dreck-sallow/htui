use crate::tools::tui::core::elements::nested_list::{
    item_v2::{NestedListItem, NestedListItemState}, state_v2::NestedListStateV2, NestedCursor,
};

#[derive(Default)]
pub struct CollectionState {
    pub list: NestedListStateV2<RequestItem, CollectionItem>,
}

impl CollectionState {
    pub fn next(&mut self) {
        self.list.next_v2(|itm| match itm {
            NestedListItem::Single(_) => true,
            NestedListItem::Group { inner, .. } => !inner.is_closed,
        });
    }

    pub fn prev(&mut self) {
        self.list.prev_v2(|itm| match itm {
            NestedListItem::Single(_) => true,
            NestedListItem::Group { inner, .. } => !inner.is_closed,
        });
    }

    pub fn close_group(&mut self) {
        if let Some(NestedListItemState::Group(inner)) = self.list.current_inner_mut() {
            inner.is_closed = true; 
        }        
    }

    
    pub fn open_group(&mut self) {
        if let Some(NestedListItemState::Group(inner)) = self.list.current_inner_mut() {
            inner.is_closed = false; 
        }
    }    

    pub fn add_item(&mut self, item: CollectionItem, sub_items: Vec<RequestItem>) {
        let children = {
            let mut list = Vec::new();

            for req in sub_items {
                list.push(NestedListItem::Single(req));
            }

            list
        };

        self.list.insert(NestedListItem::Group {
            inner: item,
            items: children,
        });
    }

    pub fn clone_item(&mut self) {
        if let Some(state) = self.list.current_inner() {
            match state {
                crate::tools::tui::core::elements::nested_list::item_v2::NestedListItemState::Single(single) => {
                    self.list.insert(NestedListItem::Single(single.clone()))
                },
                crate::tools::tui::core::elements::nested_list::item_v2::NestedListItemState::Group(_) => {
                    
                },
            }
        }
    }

    pub fn cursor(&self) -> NestedCursor {
        self.list.cursor()
    }
}

#[derive(Default, Clone)]
pub struct CollectionItem {
    pub name: String,
    pub is_closed: bool
}

impl CollectionItem {
    pub fn new<S>(name: S) -> Self
    where
        S: Into<String>,
    {
        Self { name: name.into(), is_closed: false }
    }
}

#[derive(Default, Clone)]
pub struct RequestItem {
    pub name: String,
    pub method: String,
    pub url: String,
}

impl RequestItem {
    pub fn new<S>(name: S, method: String, url: String) -> Self
    where
        S: Into<String>,
    {
        Self {
            name: name.into(),
            method,
            url,
        }
    }
}
