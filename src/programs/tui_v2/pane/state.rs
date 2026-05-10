use std::{
    collections::{HashMap, HashSet},
    ops::Not,
    path::PathBuf,
    sync::{Arc, RwLock},
    time::Duration,
};

use crate::{
    programs::tui_v2::common::list,
    store::models::{
        time_as_id, CollectionModel, Environment, FormParam, HttpMethod, KeyValueParam,
        RequestBody, RequestModel, TimeId,
    },
};

pub struct PaneState {
    pub(crate) focus: SectionFocus,
    // pub(crate) upsert_item_action: UpsertItemAction,
    pub(crate) collections: CollectionsList,
    pub(crate) environments: Environments,
    pub(crate) selected_env_context: Option<usize>,
    pub(crate) responses: Responses,
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
            responses: Responses::new(),
        }
    }

    pub fn new(list: CollectionsList, envs: Environments, selected_env: Option<usize>) -> Self {
        Self {
            focus: SectionFocus::Collections,
            collections: list,
            environments: envs,
            selected_env_context: selected_env,
            responses: Responses::new(),
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum SectionFocus {
    Collections,
    RequestBar,
    RequestBuilder,
    ResponseViewer,
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

    pub fn new() -> Self {
        Self {
            idx: ListIdx::None,
            items: Vec::new(),
        }
    }

    pub fn with_collections(mut self, items: Vec<CollectionItem>) -> Self {
        self.idx = if items.is_empty() {
            ListIdx::None
        } else if items[0].requests.is_empty() {
            ListIdx::Group(0)
        } else {
            ListIdx::Item(0, 0)
        };

        self.items = items;

        self
    }

    pub fn size(&self) -> usize {
        self.items.len()
    }

    pub fn list(&self) -> &Vec<CollectionItem> {
        &self.items
    }

    pub fn current_req(&self) -> Option<&RequestItem> {
        if let ListIdx::Item(i, sub_i) = self.idx {
            return self.items[i].requests.get(sub_i);
        }

        None
    }

    pub fn current_req_mut(&mut self) -> Option<&mut RequestItem> {
        if let ListIdx::Item(i, sub_i) = self.idx {
            return self.items[i].requests.get_mut(sub_i);
        }

        None
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

    pub fn new_v2(id: TimeId, name: String) -> Self {
        Self {
            id,
            is_open: true,
            name,
            requests: Vec::new(),
        }
    }

    pub fn with_reqs(mut self, requests: Vec<RequestItem>) -> Self {
        self.requests = requests;
        self
    }
}

pub struct RequestItem {
    id: TimeId,
    pub name: String,
    pub url: String,
    pub headers: ParamsTable,
    pub params: ParamsTable,
    pub method: HttpMethod,
    pub body: BodyContent, // pub body: RequestBody,
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
            body: BodyContent::None,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn new_v2(
        id: TimeId,
        name: String,
        url: String,
        headers: ParamsTable,
        params: ParamsTable,
        method: HttpMethod,
        body: BodyContent,
    ) -> Self {
        Self {
            id: id,
            name: name,
            url: url,
            headers,
            params,
            method,
            body,
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
            body: BodyContent::from_model(model.body),
        }
    }
}

#[derive(Clone)]
pub enum BodyContent {
    None,
    File(FileContent),
    FormUrlEncoded(ParamsTable),
    FormData(BodyForm),
    Text(String),
}

impl BodyContent {
    pub fn from_model(body: RequestBody) -> Self {
        match body {
            RequestBody::None => Self::None,
            RequestBody::Text(st) => Self::Text(st),
            RequestBody::Json(value) => Self::Text(value.to_string()),
            RequestBody::FormUrlEncoded(values) => {
                Self::FormUrlEncoded(ParamsTable::from_model(values))
            }
            RequestBody::FormData(params) => Self::FormData(BodyForm::from_model(params)),
            RequestBody::File(path) => todo!(),
        }
    }

    pub fn none() -> Self {
        Self::None
    }

    pub fn file(content: FileContent) -> Self {
        Self::File(content)
    }

    pub fn form_url(params: ParamsTable) -> Self {
        Self::FormUrlEncoded(params)
    }

    pub fn form_data(form: BodyForm) -> Self {
        Self::FormData(form)
    }

    pub fn text(txt: String) -> Self {
        Self::Text(txt)
    }
}

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

#[derive(Default, Clone)]
pub struct ParamsTable {
    pub idx: Option<(usize, usize)>,
    pub items: Vec<ParamItem>,
}

impl ParamsTable {
    pub fn new() -> Self {
        Self {
            idx: None,
            items: Vec::new(),
        }
    }

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

    pub fn current(&self) -> Option<&ParamItem> {
        self.idx.and_then(|(i, _)| self.items.get(i))
    }

    pub fn current_mut(&mut self) -> Option<&mut ParamItem> {
        self.idx.and_then(|(i, _)| self.items.get_mut(i))
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn delete(&mut self, n: usize) -> ParamItem {
        let itm = self.items.remove(n);

        if let Some((i, sub_i)) = self.idx {
            self.idx = list::clamp_index(Some(i), self.items.len()).map(|i| (i, sub_i));
        }

        itm
    }

    pub fn delete_current(&mut self) -> Option<ParamItem> {
        self.idx.and_then(|(i, _)| Some(self.delete(i)))
    }

    pub fn add_item(&mut self, itm: ParamItem) {
        self.items.push(itm);

        if self.idx.is_none() {
            self.idx = Some((0, 0));
        }
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

#[derive(Clone)]
pub struct ParamItem {
    pub enable: bool,
    pub key: String,
    pub value: String,
}

impl ParamItem {
    pub fn new(key: String, value: String) -> Self {
        Self {
            enable: true,
            key,
            value,
        }
    }

    pub fn empty() -> Self {
        Self {
            enable: true,
            key: "".into(),
            value: "".into(),
        }
    }

    // pub fn enable(&mut self) {
    //     self.enable = true;
    // }

    // pub fn disabled(&mut self) {
    //     self.enable = false;
    // }

    pub fn toggle_enable(&mut self) {
        self.enable = !self.enable;
    }
}

#[derive(Default, Clone)]
pub struct BodyForm {
    pub idx: Option<(usize, usize)>,
    pub items: Vec<ParamItem>,
    pub are_files: HashSet<usize>,
}

impl BodyForm {
    pub fn from_model(model: Vec<FormParam>) -> Self {
        let mut items = Vec::with_capacity(model.len());
        let mut files_set = HashSet::new();

        let mut i = 0;
        for param in model {
            if param.is_file {
                files_set.insert(i);
            }

            items.push(ParamItem {
                enable: true,
                key: param.key,
                value: param.value,
            });
            i += 1;
        }

        let idx = items.is_empty().not().then_some((0, 0));

        Self {
            items,
            idx,
            are_files: files_set,
        }
    }

    pub fn current_mut(&mut self) -> Option<&mut ParamItem> {
        self.idx.and_then(|(i, _)| self.items.get_mut(i))
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn delete(&mut self, n: usize) -> ParamItem {
        let itm = self.items.remove(n);

        if let Some((i, sub_i)) = self.idx {
            self.idx = list::clamp_index(Some(i), self.items.len()).map(|i| (i, sub_i));
        }

        itm
    }

    pub fn delete_current(&mut self) -> Option<ParamItem> {
        self.idx.and_then(|(i, _)| Some(self.delete(i)))
    }

    pub fn add_item(&mut self, itm: ParamItem) {
        self.items.push(itm);

        if self.idx.is_none() {
            self.idx = Some((0, 0));
        }
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

#[derive(Clone)]
pub enum FileContent {
    None,
    Content { path: PathBuf, info: FileInfo },
}

#[derive(Clone)]
pub struct FileInfo {
    pub name: String,
    pub size: String,
    pub path: String,
}

pub struct Responses {
    pub list: HashMap<TimeId, ResponseStatus>,
    pub map: Arc<RwLock<HashMap<TimeId, ResponseStatus>>>,
}

impl Responses {
    pub fn new() -> Self {
        Self {
            list: HashMap::new(),
            map: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn responses(&self) -> Arc<RwLock<HashMap<TimeId, ResponseStatus>>> {
        Arc::clone(&self.map)
    }
}

pub enum ResponseStatus {
    Fetching,
    Success(Response),
    Error(String),
    Cancelled,
}

pub struct Response {
    pub status: u16,
    pub status_text: String,
    pub version: String,
    pub duration: Duration,
    pub size_bytes: usize,
    pub content_type: String,
    pub headers: ParamsTable,
    pub cookies: ParamsTable,
    pub body: ResponseBody,
}

pub enum ResponseBody {
    Text(String),
    Binary(Vec<u8>),
    Empty,
}

pub struct Cookie {
    name: String,
    value: String,
    domain: Option<String>,
    expires: Option<String>,
    max_ge: Option<String>,
    path: Option<String>,
    http_only: Option<bool>,
    partitioned: Option<bool>,
    secure: Option<bool>,
    same_site: Option<String>,
}
