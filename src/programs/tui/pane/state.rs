use std::collections::HashSet;

use crate::store::models::{CollectionsModel, RequestModel, SendRequestId};

use super::{
    collections::state::{CollectionsState, Idx},
    responses::Responses,
};

#[derive(PartialEq, Clone, Copy)]
pub enum ElementFocus {
    Collections,
    MethodUrlBar,
    RequestBuilder,
    ResponseViewer,
}

pub struct PaneState {
    collections: CollectionsState,
    focus: ElementFocus,
    request_idx: Option<(usize, usize)>,
    responses: Responses,
}

impl PaneState {
    pub fn from_collections(collections: Vec<CollectionsModel>) -> Self {
        let collections = CollectionsState::from_list(collections);

        let request_idx = match collections.idx() {
            Idx::None => None,
            Idx::Parent(_) => None,
            Idx::Child(i, sub_i) => Some((i, sub_i)),
        };

        Self {
            collections,
            focus: ElementFocus::Collections,
            responses: Responses::new(),
            request_idx,
        }
    }
}

/// Implementation for view (&self)
impl PaneState {
    pub fn focus(&self) -> &ElementFocus {
        &self.focus
    }

    pub fn collections(&self) -> &[CollectionsModel] {
        &self.collections.collections()
    }

    pub fn is_focus(&self, element_focus: ElementFocus) -> bool {
        self.focus == element_focus
    }

    pub fn opened_collections(&self) -> HashSet<usize> {
        self.collections.openeds()
    }

    pub fn idx(&self) -> Idx {
        self.collections.idx()
    }

    pub fn is_sending_request(&self, id: SendRequestId) -> bool {
        match self.responses.get(&id) {
            Some(send_req) => match &*send_req.read().unwrap() {
                crate::store::models::SendRequest::Pending => true,
                crate::store::models::SendRequest::Finish(_) => false,
            },
            None => false,
        }
    }

    pub fn current_collection(&self) -> Option<&CollectionsModel> {
        match self.idx() {
            Idx::None => None,
            Idx::Parent(i) | Idx::Child(i, _) => self.collections.collections().get(i),
        }
    }

    pub fn current_request(&self) -> Option<&RequestModel> {
        self.collections.current_request()
    }
}

/// Implementation for mutations (&mut self)
impl PaneState {
    pub fn next_idx(&mut self) {
        self.collections.next();
    }

    pub fn prev_idx(&mut self) {
        self.collections.prev();
    }

    pub fn set_focus(&mut self, element_focus: ElementFocus) {
        self.focus = element_focus;
    }

    pub fn open_collection(&mut self) {
        self.collections.open_collection(true);
    }

    pub fn close_collection(&mut self) {
        self.collections.close_collection(true);
    }

    pub fn add_collection(&mut self, collection: CollectionsModel) {
        self.collections.add_collection(collection);
    }

    pub fn remove_collection(&mut self, idx: usize) -> Option<CollectionsModel> {
        self.collections.remove_collection_by_idx(idx)
    }

    pub fn current_collection_mut(&mut self, idx: usize) -> Option<&mut CollectionsModel> {
        self.collections.collections_mut().get_mut(idx)
    }

    pub fn add_request(&mut self, request: RequestModel) {
        self.collections.add_request(request);
    }

    pub fn remove_request(&mut self, idx: (usize, usize)) -> Option<RequestModel> {
        self.collections.remove_request_by_idx(idx)
    }

    pub fn edit_request<F: FnMut(&mut RequestModel)>(&mut self, idx: (usize, usize), mut f: F) {
        if let Some(req) = self
            .collections
            .collections_mut()
            .get_mut(idx.0)
            .and_then(|coll| coll.get_request_mut(idx.1))
        {
            f(req)
        }
    }
}
