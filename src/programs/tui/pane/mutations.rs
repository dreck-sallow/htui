use std::collections::HashMap;

use crate::store::models::{BodyContent, CollectionsModel, HttpMethod, RequestModel};

use super::{mutation_history::Mutation, state::ElementFocus};

pub enum Movement {
    Next,
    Previous,
}

pub struct SetIdx {
    movement: Movement,
}

impl SetIdx {
    pub fn for_next() -> Self {
        Self {
            movement: Movement::Next,
        }
    }

    pub fn for_previous() -> Self {
        Self {
            movement: Movement::Previous,
        }
    }
}

impl Mutation for SetIdx {
    fn apply(
        &mut self,
        state: &mut super::state::PaneState,
        _sender_event: &crate::programs::tui::events::EventSender,
    ) {
        match self.movement {
            Movement::Next => state.next_idx(),
            Movement::Previous => state.prev_idx(),
        }
    }

    fn undo(
        &mut self,
        state: &mut super::state::PaneState,
        _sender_event: &crate::programs::tui::events::EventSender,
    ) {
        match self.movement {
            Movement::Next => state.prev_idx(),
            Movement::Previous => state.next_idx(),
        }
    }
}

/// Mutation for set the focus section on the ui
pub struct SetFocus {
    navigation: Movement,
}

impl SetFocus {
    pub fn for_next() -> Self {
        Self {
            navigation: Movement::Next,
        }
    }

    pub fn for_previous() -> Self {
        Self {
            navigation: Movement::Previous,
        }
    }

    fn next_focus(focus: &ElementFocus) -> ElementFocus {
        match focus {
            ElementFocus::Collections => ElementFocus::MethodUrlBar,
            ElementFocus::MethodUrlBar => ElementFocus::RequestBuilder,
            ElementFocus::RequestBuilder => ElementFocus::ResponseViewer,
            ElementFocus::ResponseViewer => ElementFocus::Collections,
        }
    }

    fn prev_focus(focus: &ElementFocus) -> ElementFocus {
        match focus {
            ElementFocus::Collections => ElementFocus::ResponseViewer,
            ElementFocus::MethodUrlBar => ElementFocus::Collections,
            ElementFocus::RequestBuilder => ElementFocus::MethodUrlBar,
            ElementFocus::ResponseViewer => ElementFocus::RequestBuilder,
        }
    }
}

impl Mutation for SetFocus {
    fn apply(
        &mut self,
        state: &mut super::state::PaneState,
        _sender_event: &crate::programs::tui::events::EventSender,
    ) {
        match self.navigation {
            Movement::Next => state.set_focus(Self::next_focus(state.focus())),
            Movement::Previous => state.set_focus(Self::prev_focus(state.focus())),
        }
    }

    fn undo(
        &mut self,
        state: &mut super::state::PaneState,
        _sender_event: &crate::programs::tui::events::EventSender,
    ) {
        match self.navigation {
            Movement::Next => state.set_focus(Self::prev_focus(state.focus())),
            Movement::Previous => state.set_focus(Self::next_focus(state.focus())),
        }
    }
}

// Collection mutations
pub struct OpenCloseCollection(bool);

impl OpenCloseCollection {
    pub fn for_close() -> Self {
        Self(false)
    }

    pub fn for_open() -> Self {
        Self(true)
    }
}

impl Mutation for OpenCloseCollection {
    fn apply(
        &mut self,
        state: &mut super::state::PaneState,
        _sender_event: &crate::programs::tui::events::EventSender,
    ) {
        if self.0 {
            state.open_collection();
        } else {
            state.close_collection();
        }

        self.0 = !self.0;
    }

    fn undo(
        &mut self,
        state: &mut super::state::PaneState,
        _sender_event: &crate::programs::tui::events::EventSender,
    ) {
        self.apply(state, _sender_event);
    }
}

pub struct CreateCollection {
    collection: Option<CollectionsModel>,
}

impl CreateCollection {
    pub fn new(collection: CollectionsModel) -> Self {
        Self {
            collection: Some(collection),
        }
    }
}

impl Mutation for CreateCollection {
    fn apply(
        &mut self,
        state: &mut super::state::PaneState,
        _sender_event: &crate::programs::tui::events::EventSender,
    ) {
        if let Some(coll) = self.collection.take() {
            state.add_collection(coll);
        }
    }

    fn undo(
        &mut self,
        state: &mut super::state::PaneState,
        _sender_event: &crate::programs::tui::events::EventSender,
    ) {
        self.collection = state.remove_collection(state.collections().len() - 1)
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

impl Mutation for RemoveCollection {
    fn apply(
        &mut self,
        state: &mut super::state::PaneState,
        _sender_event: &crate::programs::tui::events::EventSender,
    ) {
        self.removed = state.remove_collection(self.idx);
    }

    fn undo(
        &mut self,
        state: &mut super::state::PaneState,
        _sender_event: &crate::programs::tui::events::EventSender,
    ) {
        if let Some(coll) = self.removed.take() {
            // TODO: use the idx for insert, instead of append
            state.add_collection(coll);
        }
    }
}

pub struct EditCollectionName {
    idx: usize,
    swap_name: String,
}

impl EditCollectionName {
    pub fn new(idx: usize, name: String) -> Self {
        Self {
            idx,
            swap_name: name,
        }
    }
}

impl Mutation for EditCollectionName {
    fn apply(
        &mut self,
        state: &mut super::state::PaneState,
        _sender_event: &crate::programs::tui::events::EventSender,
    ) {
        let collection = state.current_collection_mut(self.idx).unwrap();
        let temp_name = collection.name.clone();
        collection.name = self.swap_name.clone();
        self.swap_name = temp_name;
    }

    fn undo(
        &mut self,
        state: &mut super::state::PaneState,
        _sender_event: &crate::programs::tui::events::EventSender,
    ) {
        self.apply(state, _sender_event);
    }
}

/// Request mutations
///

pub struct CreateRequest {
    coll_i: usize,
    request: Option<RequestModel>,
}

impl CreateRequest {
    pub fn new(collection_idx: usize, req: RequestModel) -> Self {
        Self {
            request: Some(req),
            coll_i: collection_idx,
        }
    }
}

impl Mutation for CreateRequest {
    fn apply(
        &mut self,
        state: &mut super::state::PaneState,
        _sender_event: &crate::programs::tui::events::EventSender,
    ) {
        if let Some(req) = self.request.take() {
            state.add_request(req);
        }
    }

    fn undo(
        &mut self,
        state: &mut super::state::PaneState,
        _sender_event: &crate::programs::tui::events::EventSender,
    ) {
        if let Some(coll) = state.collections().get(self.coll_i) {
            self.request = state.remove_request((self.coll_i, coll.requests.len() - 1));
        }
    }
}

#[derive(Clone)]
pub enum RequestEditType {
    Name(String),
    Url(String),
    Method(HttpMethod),
    Headers(HashMap<String, String>),
    Body(BodyContent),
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

impl Mutation for EditRequest {
    fn apply(
        &mut self,
        state: &mut super::state::PaneState,
        _sender_event: &crate::programs::tui::events::EventSender,
    ) {
        let req = &state.collections()[self.idx.0].requests()[self.idx.1];

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
            RequestEditType::Body(body_content) => {
                self.previous_action = RequestEditType::Body(req.body().clone());
                state.edit_request(self.idx, |req| {
                    req.set_body(body_content.clone());
                });
            }
        }
    }

    fn undo(
        &mut self,
        state: &mut super::state::PaneState,
        _sender_event: &crate::programs::tui::events::EventSender,
    ) {
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
            RequestEditType::Body(body_content) => {
                state.edit_request(self.idx, |req| {
                    req.set_body(body_content.clone());
                });
            }
        }
    }
}

pub struct RemoveRequest {
    idx: (usize, usize),
    removed: Option<RequestModel>,
}

impl RemoveRequest {
    pub fn new(idx: (usize, usize)) -> Self {
        Self { idx, removed: None }
    }
}

impl Mutation for RemoveRequest {
    fn apply(
        &mut self,
        state: &mut super::state::PaneState,
        _sender_event: &crate::programs::tui::events::EventSender,
    ) {
        self.removed = state.remove_request(self.idx);
    }

    fn undo(
        &mut self,
        state: &mut super::state::PaneState,
        _sender_event: &crate::programs::tui::events::EventSender,
    ) {
        if let Some(req) = self.removed.take() {
            // TODO: use the idx for insert, instead of append
            state.add_request(req);
        }
    }
}
