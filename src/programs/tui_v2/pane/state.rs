use crate::store::models::{
    CollectionModel, Environment, HttpMethod, KeyValueParam, RequestModel, TimeId,
};

pub struct PaneState {
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
            collections: CollectionsList::from_model(collections),
            environments: Environments::from_model(environments),
            selected_env_context: selected_env,
        }
    }
}

pub struct CollectionsList {
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
                is_open: false,
                name: collection.name,
                requests,
            });
        }

        Self { items }
    }
}

impl CollectionsList {
    pub fn size(&self) -> usize {
        self.items.len()
    }

    pub fn list(&self) -> &Vec<CollectionItem> {
        &self.items
    }
}

pub struct CollectionItem {
    id: TimeId,
    pub is_open: bool,
    pub name: String,
    pub requests: Vec<RequestItem>,
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

pub struct ParamsTable {
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

        Self { items }
    }
}

pub struct ParamItem {
    pub enable: bool,
    pub key: String,
    pub value: String,
}
