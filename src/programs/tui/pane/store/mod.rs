use collections_store::{CollectionsStore, MutableList};
use responses_store::Responses;

use crate::store::models::{CollectionsModel, RequestModel, SendRequestKey};

use super::{responses::SendRequestResponse, state::ElementFocus};

pub mod actions;
mod collections_store;
pub mod mutator;
mod responses_store;

pub struct PaneStore {
    // collections: CollectionsStore,
    current_request_idx: Option<(usize, usize)>,
    focus: ElementFocus,
    responses: Responses,
}

impl PaneStore {
    pub fn new() -> Self {
        Self {
            current_request_idx: None,
            // collections: CollectionsStore::new(collections),
            focus: ElementFocus::Collections,
            responses: Responses::new(),
        }
    }

    // pub fn collections(&self) -> &[CollectionsModel] {
    //     self.collections.list()
    // }

    pub fn is_focus(&self, focus: ElementFocus) -> bool {
        self.focus == focus
    }

    pub fn focus(&self) -> ElementFocus {
        self.focus
    }

    pub fn current_request_idx(&self) -> Option<(usize, usize)> {
        self.current_request_idx
    }

    // pub fn current_request(&self) -> Option<&RequestModel> {
    //     self.current_request_idx
    //         .and_then(|idx| self.collections.get_request(idx))
    // }

    pub fn is_sending_request(&self, id: SendRequestKey) -> bool {
        match self.responses.get(&id) {
            Some(req) => match &*req.read().unwrap() {
                crate::store::models::SendRequest::Pending => true,
                crate::store::models::SendRequest::Finish(_) => false,
            },
            None => false,
        }
    }

    pub fn current_send_request(&self, id: SendRequestKey) -> Option<SendRequestResponse> {
        self.responses.get(&id)
    }
}

impl PaneStore {
    pub fn set_focus(&mut self, focus: ElementFocus) {
        self.focus = focus;
    }

    pub fn set_current_idx(&mut self, idx: Option<(usize, usize)>) {
        self.current_request_idx = idx;
    }
}
