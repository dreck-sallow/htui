use crate::tools::tui::core::elements::nested_list::{
    // state::{NestedListItem, NestedListState},
    item::{NestedListMultiple, NestedListSingle},
    state_v2::{NestedListItem, NestedListStateV2},
    NestedCursor,
};

#[derive(Default)]
pub struct CollectionState {
    pub list: NestedListStateV2<RequestItem, CollectionItem>,
}

impl CollectionState {
    pub fn add_item(&mut self, item: CollectionItem, sub_items: Vec<RequestItem>) {
        let children = {
            let mut list = Vec::new();

            for req in sub_items {
                list.push(NestedListSingle(req));
            }

            list
        };

        let multiple = NestedListMultiple::new(item).with_children(children);

        self.list.insert(NestedListItem::Multiple(multiple));
    }

    pub fn clone_item(&mut self) {
        if let Some(idx) = self.list.cursor().idx() {
            match &self.list.items()[*idx] {
                NestedListItem::Sigle(ref single) => {
                    self.list.insert(NestedListItem::Sigle(single.clone()))
                }
                NestedListItem::Multiple(multiple) => match self.list.cursor().sub_idx() {
                    Some(sub_idx) => self
                        .list
                        .insert_on_multiple(multiple.child(*sub_idx).clone(), *idx),
                    None => self.list.insert(NestedListItem::Multiple(multiple.clone())),
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
}

impl CollectionItem {
    pub fn new<S>(name: S) -> Self
    where
        S: Into<String>,
    {
        Self { name: name.into() }
    }
}

#[derive(Default, Clone)]
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
