use std::{collections::HashSet, path::PathBuf};

use crate::store::models::{time_as_id, HttpMethod, TimeId};

use super::{
    param_item::ParamItem,
    table::{TableIdx, TableState},
};

#[derive(Default, Debug, Clone, PartialEq, Eq, Copy)]
pub enum ListIdx {
    #[default]
    None,
    Group(usize),
    Item(usize, usize),
}

pub struct CollectionsList {
    pub idx: ListIdx,
    items: Vec<CollectionItem>,
}

impl CollectionsList {
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

    pub fn items(&self) -> &[CollectionItem] {
        &self.items
    }

    pub fn add_item(&mut self, itm: CollectionItem) {
        self.items.push(itm);
    }

    pub fn insert_item(&mut self, idx: usize, itm: CollectionItem) {
        self.items.insert(idx, itm);
    }

    pub fn insert_req(&mut self, (coll_i, req_i): (usize, usize), req: RequestItem) {
        self.items[coll_i].requests.insert(req_i, req);
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

    pub fn get_collection(&self, i: usize) -> Option<&CollectionItem> {
        self.items.get(i)
    }

    pub fn get_mut_collection(&mut self, i: usize) -> Option<&mut CollectionItem> {
        self.items.get_mut(i)
    }

    pub fn remove_collection(&mut self, i: usize) {
        self.items.remove(i);
    }

    pub fn get_requests(&self, i: usize) -> Option<&Vec<RequestItem>> {
        self.items.get(i).and_then(|coll| Some(&coll.requests))
    }

    pub fn get_mut_requests(&mut self, i: usize) -> Option<&mut Vec<RequestItem>> {
        self.items
            .get_mut(i)
            .and_then(|coll| Some(&mut coll.requests))
    }

    pub fn get_request(&self, (i, sub_i): (usize, usize)) -> Option<&RequestItem> {
        self.items.get(i).and_then(|coll| coll.requests.get(sub_i))
    }

    pub fn get_request_mut(&mut self, (i, sub_i): (usize, usize)) -> Option<&mut RequestItem> {
        self.items
            .get_mut(i)
            .and_then(|coll| coll.requests.get_mut(sub_i))
    }
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
    pub headers: TableState<ParamItem>,
    pub params: TableState<ParamItem>,
    pub method: HttpMethod,
    pub body: BodyContent, // pub body: RequestBody,
}

impl RequestItem {
    pub fn new(name: String) -> Self {
        Self {
            id: time_as_id(),
            name,
            url: "".to_string(),
            headers: TableState::new(),
            params: TableState::new(),
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
        headers: TableState<ParamItem>,
        params: TableState<ParamItem>,
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

    // pub fn from_model(model: RequestModel) -> Self {
    //     Self {
    //         id: model.id,
    //         name: model.name,
    //         url: model.url,
    //         headers: ParamsTable::from_model(model.headers),
    //         params: ParamsTable::from_model(model.params),
    //         method: model.method,
    //         body: BodyContent::from_model(model.body),
    //     }
    // }
}

#[derive(Clone)]
pub enum BodyContent {
    None,
    File(FileContent),
    FormUrlEncoded(TableState<ParamItem>),
    FormData(BodyForm),
    Text(String),
}

impl BodyContent {
    // pub fn from_model(body: RequestBody) -> Self {
    //     match body {
    //         RequestBody::None => Self::None,
    //         RequestBody::Text(st) => Self::Text(st),
    //         RequestBody::Json(value) => Self::Text(value.to_string()),
    //         RequestBody::FormUrlEncoded(values) => {
    //             Self::FormUrlEncoded(ParamsTable::from_model(values))
    //         }
    //         RequestBody::FormData(params) => Self::FormData(BodyForm::from_model(params)),
    //         RequestBody::File(path) => todo!(),
    //     }
    // }

    pub fn none() -> Self {
        Self::None
    }

    pub fn file(content: FileContent) -> Self {
        Self::File(content)
    }

    // pub fn form_url(params: ParamsTable) -> Self {
    //     Self::FormUrlEncoded(params)
    // }

    // pub fn form_data(form: BodyForm) -> Self {
    //     Self::FormData(form)
    // }

    pub fn text(txt: String) -> Self {
        Self::Text(txt)
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

#[derive(Clone)]
pub struct BodyForm {
    inner: TableState<ParamItem>,
    pub are_files: HashSet<usize>,
}

impl BodyForm {
    pub fn new() -> Self {
        Self {
            inner: TableState::new(),
            are_files: HashSet::new(),
        }
    }

    pub fn current_mut(&mut self) -> Option<&mut ParamItem> {
        self.inner.current_mut()
    }

    pub fn idx(&self) -> TableIdx {
        self.inner.idx()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn items(&self) -> &[ParamItem] {
        self.inner.items()
    }

    pub fn delete(&mut self, n: usize) -> Option<ParamItem> {
        self.inner.remove_row(n)
    }

    pub fn remove_current(&mut self) -> Option<ParamItem> {
        self.inner.remove_current_row()
    }

    pub fn add_item(&mut self, itm: ParamItem) {
        self.inner.add_row(itm);
    }

    pub fn next_row(&mut self) {
        self.inner.next_row();
    }

    pub fn prev_row(&mut self) {
        self.inner.prev_row();
    }

    pub fn next_col(&mut self) {
        self.inner.next_cell();
    }

    pub fn prev_col(&mut self) {
        self.inner.prev_cell();
    }
}

impl Default for BodyForm {
    fn default() -> Self {
        Self {
            inner: TableState::new(),
            are_files: HashSet::new(),
        }
    }
}

impl From<Vec<(bool, ParamItem)>> for BodyForm {
    fn from(value: Vec<(bool, ParamItem)>) -> Self {
        let mut table = TableState::new();
        let mut are_files = HashSet::new();

        for (is_file, param) in value {
            if is_file {
                are_files.insert(table.len());
            }

            table.add_row(param);
        }

        Self {
            inner: table,
            are_files,
        }
    }
}

// impl BodyForm {
//     pub fn from_model(model: Vec<FormParam>) -> Self {
//         let mut items = Vec::with_capacity(model.len());
//         let mut files_set = HashSet::new();

//         let mut i = 0;
//         for param in model {
//             if param.is_file {
//                 files_set.insert(i);
//             }

//             items.push(ParamItem {
//                 enable: true,
//                 key: param.key,
//                 value: param.value,
//             });
//             i += 1;
//         }

//         let idx = items.is_empty().not().then_some((0, 0));

//         Self {
//             items,
//             idx,
//             are_files: files_set,
//         }
//     }
// }
