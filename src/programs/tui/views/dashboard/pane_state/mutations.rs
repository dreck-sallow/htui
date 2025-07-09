use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use reqwest::{ClientBuilder, Url};
use tokio::{sync::mpsc, time::Instant};

use crate::{
    programs::tui::{events::Event, views::dashboard::focus::ElementFocus},
    store::models::{
        BodyContent, CollectionsModel, HttpMethod, RequestModel, ResponseModel, SendRequest,
        SendRequestId,
    },
};

use super::{
    responses::{RequestTask, SendRequestResponse},
    PaneState,
};

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

pub enum FocusNavigation {
    Next,
    Prev,
}

pub struct SetFocus {
    navigation: FocusNavigation,
}

impl SetFocus {
    pub fn new(navigation: FocusNavigation) -> Self {
        Self { navigation }
    }

    pub fn for_next() -> Self {
        Self::new(FocusNavigation::Next)
    }

    pub fn for_previous() -> Self {
        Self::new(FocusNavigation::Prev)
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

impl PaneStateMutation for SetFocus {
    fn apply(&mut self, state: &mut PaneState) {
        match self.navigation {
            FocusNavigation::Next => state.focus_element(Self::next_focus(&state.element_focus)),
            FocusNavigation::Prev => state.focus_element(Self::prev_focus(&state.element_focus)),
        }
    }

    fn undo(&mut self, state: &mut PaneState) {
        match self.navigation {
            FocusNavigation::Next => state.focus_element(Self::prev_focus(&state.element_focus)),
            FocusNavigation::Prev => state.focus_element(Self::next_focus(&state.element_focus)),
        }
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

impl RemoveRequest {
    pub fn new(idx: (usize, usize)) -> Self {
        Self { idx, removed: None }
    }
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
            RequestEditType::Body(body_content) => {
                self.previous_action = RequestEditType::Body(req.body().clone());
                state.edit_request(self.idx, |req| {
                    req.set_body(body_content.clone());
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
            RequestEditType::Body(body_content) => {
                state.edit_request(self.idx, |req| {
                    req.set_body(body_content.clone());
                });
            }
        }
    }
}

pub struct SendRequestMutation {
    send_request_id: SendRequestId,
}

impl SendRequestMutation {
    pub fn new(id: SendRequestId) -> Self {
        Self {
            send_request_id: id,
        }
    }

    pub fn send(
        &self,
        req: &RequestModel,
        send_request: SendRequestResponse,
        send_events: mpsc::UnboundedSender<Event>,
    ) -> RequestTask {
        let (tx, tr) = tokio::sync::oneshot::channel();

        let url = Url::parse(req.url()).unwrap();
        let method = match req.method() {
            HttpMethod::Options => reqwest::Method::OPTIONS,
            HttpMethod::Get => reqwest::Method::GET,
            HttpMethod::Post => reqwest::Method::POST,
            HttpMethod::Put => reqwest::Method::PUT,
            HttpMethod::Delete => reqwest::Method::DELETE,
            HttpMethod::Head => reqwest::Method::HEAD,
            HttpMethod::Patch => reqwest::Method::PATCH,
        };

        let mut client_builder = ClientBuilder::new()
            .referer(false)
            .build()
            .unwrap()
            .request(method, url);

        for (key, value) in req.headers_map() {
            client_builder = client_builder.header(key, value);
        }

        let jh = tokio::spawn(async move {
            let timer = Instant::now();
            tokio::select! {
                result = client_builder.send() => {
                    match result {
                        Ok(res) => {
                            let response = ResponseModel { duration: timer.elapsed(),  status: res.status().as_u16(), body: "BODY", headers: HashMap::new() };
                            *send_request.write().unwrap() = SendRequest::Finish(response);
                        }
                        Err(_e) => {
                        }
                    }
                }
                _ = tr => {
                }
            }
            let _ = send_events.send(Event::Draw);
        });

        RequestTask::new(tx, jh)
    }
}

impl PaneStateMutation for SendRequestMutation {
    fn apply(&mut self, state: &mut PaneState) {
        let send_req = match state.responses.stop(&self.send_request_id) {
            Some(req) => {
                *req.write().unwrap() = SendRequest::Pending;
                req
            }
            None => Arc::new(RwLock::new(SendRequest::Pending)),
        };

        let current_req = state
            .project
            .request_by_idx(state.current_request_idx.unwrap())
            .unwrap();

        let request_task = self.send(
            current_req,
            Arc::clone(&send_req),
            state.__send_event.clone(),
        );

        state
            .responses
            .add(self.send_request_id.clone(), send_req, request_task);
    }

    fn undo(&mut self, _state: &mut PaneState) {}
}
