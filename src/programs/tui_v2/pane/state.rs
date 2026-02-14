use std::ops::Not;

use crate::{
    programs::tui_v2::common::list,
    store::models::{
        time_as_id, CollectionModel, Environment, HttpMethod, KeyValueParam, RequestModel, TimeId,
    },
};

pub struct PaneState {
    pub(crate) focus: SectionFocus,
    // pub(crate) upsert_item_action: UpsertItemAction,
    pub(crate) collections: CollectionsList,
    pub(crate) environments: Environments,
    pub(crate) selected_env_context: Option<usize>,
}

impl PaneState {
    pub fn from_parts(
        collections: Vec<CollectionModel>,
        environments: Vec<Environment>,
        selected_env: Option<usize>,
    ) -> Self {
        Self {
            focus: SectionFocus::Collections,
            // upsert_item_action: UpsertItemAction::CreateCollection,
            collections: CollectionsList::from_model(collections),
            environments: Environments::from_model(environments),
            selected_env_context: selected_env,
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum SectionFocus {
    Collections,
    RequestBar,
    RequestBuilder,
}

pub struct CollectionsList {
    pub idx: ListIdx,
    pub items: Vec<CollectionItem>,
}

impl CollectionsList {
    pub fn from_model(model: Vec<CollectionModel>) -> Self {
        let mut items = Vec::with_capacity(model.len());

        for collection in model {
            let mut requests = Vec::with_capacity(collection.requests.len());
            for req in collection.requests {
                requests.push(RequestItem::from_model(req));
            }

            items.push(CollectionItem {
                id: collection.id,
                is_open: true,
                name: collection.name,
                requests,
            });
        }

        let idx = if items.is_empty() {
            ListIdx::None
        } else if items[0].requests.is_empty() {
            ListIdx::Group(0)
        } else {
            ListIdx::Item(0, 0)
        };

        Self { items, idx }
    }
    pub fn size(&self) -> usize {
        self.items.len()
    }

    pub fn list(&self) -> &Vec<CollectionItem> {
        &self.items
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Copy)]
pub enum ListIdx {
    #[default]
    None,
    Group(usize),
    Item(usize, usize),
}

pub struct CollectionItem {
    id: TimeId,
    pub is_open: bool,
    pub name: String,
    pub requests: Vec<RequestItem>,
}

impl CollectionItem {
    pub fn new(name: String) -> Self {
        Self {
            id: time_as_id(),
            is_open: false,
            name,
            requests: Vec::new(),
        }
    }
}

pub struct RequestItem {
    id: TimeId,
    pub name: String,
    pub url: String,
    pub headers: ParamsTable,
    pub params: ParamsTable,
    pub method: HttpMethod,
    // pub body: RequestBody,
}

impl RequestItem {
    pub fn new(name: String) -> Self {
        Self {
            id: time_as_id(),
            name,
            url: "".to_string(),
            headers: ParamsTable::default(),
            params: ParamsTable::default(),
            method: HttpMethod::Get,
        }
    }

    pub fn from_model(model: RequestModel) -> Self {
        Self {
            id: model.id,
            name: model.name,
            url: model.url,
            headers: ParamsTable::from_model(model.headers),
            params: ParamsTable::from_model(model.params),
            method: model.method,
        }
    }
}

// pub struct KeyValueParam {
//     pub enable: bool,
//     pub key: String,
//     pub value: String,
// }

pub struct Environments {
    items: Vec<EnvironmentItem>,
}

impl Environments {
    pub fn from_model(model: Vec<Environment>) -> Self {
        let mut items = Vec::with_capacity(model.len());

        for env in model {
            items.push(EnvironmentItem {
                name: env.name,
                variables: ParamsTable::from_model(Vec::new()),
            });
        }

        Self { items }
    }
}

pub struct EnvironmentItem {
    pub name: String,
    pub variables: ParamsTable,
}

pub struct Variable {
    pub name: String,
    pub value: String,
}

#[derive(Default)]
pub struct ParamsTable {
    pub idx: Option<(usize, usize)>,
    pub items: Vec<ParamItem>,
}

impl ParamsTable {
    pub fn from_model(model: Vec<KeyValueParam>) -> Self {
        let mut items = Vec::with_capacity(model.len());

        for param in model {
            items.push(ParamItem {
                enable: true,
                key: param.key,
                value: param.value,
            });
        }

        let idx = items.is_empty().not().then_some((0, 0));
        Self { items, idx }
    }

    pub fn next_row(&mut self) {
        let row = list::next(self.idx.map(|(i, _)| i), self.items.len());
        self.idx = row.map(|i| (i, self.idx.map(|(_, i)| i).unwrap_or(0)));
    }

    pub fn prev_row(&mut self) {
        let row = list::prev(self.idx.map(|(i, _)| i));
        self.idx = row.map(|i| (i, self.idx.map(|(_, i)| i).unwrap_or(0)));
    }

    pub fn next_col(&mut self) {
        if let Some((row, col)) = self.idx {
            if col < 2 {
                self.idx = Some((row, col + 1));
            }
        }
    }

    pub fn prev_col(&mut self) {
        if let Some((row, col)) = self.idx {
            if col > 0 {
                self.idx = Some((row, col - 1));
            }
        }
    }
}

pub struct ParamItem {
    pub enable: bool,
    pub key: String,
    pub value: String,
}

impl ParamItem {
    pub fn enable(&mut self) {
        self.enable = true;
    }

    pub fn disabled(&mut self) {
        self.enable = false;
    }
}
