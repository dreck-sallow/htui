use std::collections::HashMap;

use crate::store::models::{CollectionsModel, HttpMethod, RequestModel};

use super::PaneState;

// TODO:  For previous values, I should use Cow

pub trait PaneStateMutation {
    fn apply(&mut self, state: &mut PaneState);
    fn undo(&mut self, state: &mut PaneState);
}

pub struct SetRquestIdx {
    previous_idx: Option<(usize, usize)>,
    new_idx: Option<(usize, usize)>,
}

impl SetRquestIdx {
    pub fn new(idx: Option<(usize, usize)>) -> Self {
        Self {
            previous_idx: None,
            new_idx: idx,
        }
    }
}

impl PaneStateMutation for SetRquestIdx {
    fn apply(&mut self, state: &mut PaneState) {
        self.previous_idx = state.current_request_idx;
        state.set_request_idx(self.new_idx);
    }

    fn undo(&mut self, state: &mut PaneState) {
        state.set_request_idx(self.previous_idx);
    }
}

pub struct RemoveCollection {
    idx: usize,
    removed: Option<CollectionsModel>,
}

impl RemoveCollection {
    pub fn new(idx: usize) -> Self {
        Self { idx, removed: None }
    }
}

impl PaneStateMutation for RemoveCollection {
    fn apply(&mut self, state: &mut PaneState) {
        self.removed = Some(state.delete_collection(self.idx));
    }

    fn undo(&mut self, state: &mut PaneState) {
        if let Some(coll) = self.removed.take() {
            // FIXME: create method for insert
            state.add_collection(coll);
        }
    }
}

// FIXME: use a method for insert, to retrive the idx
pub struct AddCollection {
    collection_id: String,
    request: Option<CollectionsModel>,
}

impl AddCollection {
    pub fn new(collection: CollectionsModel) -> Self {
        Self {
            collection_id: collection.id().to_string(),
            request: Some(collection),
        }
    }
}

impl PaneStateMutation for AddCollection {
    fn apply(&mut self, state: &mut PaneState) {
        state.add_collection(self.request.take().unwrap());
    }

    fn undo(&mut self, state: &mut PaneState) {
        let idx_op = state
            .project
            .collections()
            .iter()
            .enumerate()
            .find(|(_i, coll)| coll.id() == &self.collection_id)
            .map(|(i, _)| i);

        if let Some(idx) = idx_op {
            self.request = Some(state.delete_collection(idx));
        }
    }
}

pub struct RemoveRequest {
    idx: (usize, usize),
    removed: Option<RequestModel>,
}

impl PaneStateMutation for RemoveRequest {
    fn apply(&mut self, state: &mut PaneState) {
        self.removed = state.delete_request(self.idx);
    }

    fn undo(&mut self, state: &mut PaneState) {
        if let Some(req) = self.removed.take() {
            state.add_request(self.idx.0, req);
        }
    }
}

pub struct AddRequest {
    collection_idx: usize,
    request_id: String,
    request: Option<RequestModel>,
}

impl AddRequest {
    pub fn new(collection_idx: usize, request: RequestModel) -> Self {
        Self {
            collection_idx,
            request_id: request.id().to_string(),
            request: Some(request),
        }
    }
}

impl PaneStateMutation for AddRequest {
    fn apply(&mut self, state: &mut PaneState) {
        state.add_request(self.collection_idx, self.request.take().unwrap());
    }

    fn undo(&mut self, state: &mut PaneState) {
        let idx_op = state
            .project
            .collection_by_idx(self.collection_idx)
            .unwrap()
            .requests()
            .iter()
            .enumerate()
            .find(|(_i, req)| req.id() == self.request_id)
            .map(|(i, _)| i);

        if let Some(idx) = idx_op {
            self.request = state.delete_request((self.collection_idx, idx));
        }
    }
}

pub struct EditCollectionName {
    collection_idx: usize,
    name: String,
    previous_name: String,
}

impl EditCollectionName {
    pub fn new(name: String, idx: usize) -> Self {
        Self {
            collection_idx: idx,
            name,
            previous_name: String::new(),
        }
    }
}

impl PaneStateMutation for EditCollectionName {
    fn apply(&mut self, state: &mut PaneState) {
        self.previous_name = state.project.collections()[self.collection_idx]
            .name()
            .to_string();
        state.edit_collection_name(self.collection_idx, self.name.clone());
    }

    fn undo(&mut self, state: &mut PaneState) {
        state.edit_collection_name(self.collection_idx, self.previous_name.clone());
    }
}

#[derive(Clone)]
pub enum RequestEditType {
    Name(String),
    Url(String),
    Method(HttpMethod),
    Headers(HashMap<String, String>),
}

pub struct EditRequest {
    idx: (usize, usize),
    action: RequestEditType,
    previous_action: RequestEditType,
}

impl EditRequest {
    pub fn new(idx: (usize, usize), action: RequestEditType) -> Self {
        Self {
            idx,
            previous_action: action.clone(),
            action,
        }
    }
}

impl PaneStateMutation for EditRequest {
    fn apply(&mut self, state: &mut PaneState) {
        let req = &state.project.collections()[self.idx.0].requests()[self.idx.1];

        match &self.action {
            RequestEditType::Name(name) => {
                self.previous_action = RequestEditType::Name(req.name().to_string());

                state.edit_request(self.idx, |req| {
                    req.set_name(name.clone());
                });
            }
            RequestEditType::Url(url) => {
                self.previous_action = RequestEditType::Url(req.url().to_string());

                state.edit_request(self.idx, |req| {
                    req.set_url(url.clone());
                });
            }
            RequestEditType::Method(http_method) => {
                self.previous_action = RequestEditType::Method(req.method());

                state.edit_request(self.idx, |req| {
                    req.set_method(*http_method);
                });
            }
            RequestEditType::Headers(hash_map) => {
                self.previous_action = RequestEditType::Headers(req.headers_map().clone());

                state.edit_request(self.idx, |req| {
                    req.set_headers(hash_map.clone());
                });
            }
        }
    }

    fn undo(&mut self, state: &mut PaneState) {
        match &self.previous_action {
            RequestEditType::Name(name) => {
                state.edit_request(self.idx, |req| {
                    req.set_name(name.clone());
                });
            }
            RequestEditType::Url(url) => {
                state.edit_request(self.idx, |req| {
                    req.set_url(url.clone());
                });
            }
            RequestEditType::Method(http_method) => {
                state.edit_request(self.idx, |req| {
                    req.set_method(*http_method);
                });
            }
            RequestEditType::Headers(hash_map) => {
                state.edit_request(self.idx, |req| {
                    req.set_headers(hash_map.clone());
                });
            }
        }
    }
}
