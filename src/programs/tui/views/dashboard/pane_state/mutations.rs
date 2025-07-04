use crate::store::models::{CollectionsModel, RequestModel};

use super::PaneState;

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
