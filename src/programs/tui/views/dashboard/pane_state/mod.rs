use responses::Responses;
use tokio::sync::mpsc;

use crate::{
    programs::tui::events::Event,
    store::models::{CollectionsModel, ProjectModel, RequestModel, SendRequestId},
};

use super::focus::ElementFocus;

pub mod history;
pub mod mutations;
pub mod responses;

pub struct PaneState {
    project: ProjectModel,
    element_focus: ElementFocus,
    current_request_idx: Option<(usize, usize)>,
    responses: Responses,
    __send_event: mpsc::UnboundedSender<Event>,
}

impl<'a> PaneState {
    pub fn new(project: ProjectModel, __send_event: mpsc::UnboundedSender<Event>) -> Self {
        let request_idx =
            if project.collections().is_empty() || project.collections()[0].requests().is_empty() {
                None
            } else {
                Some((0, 0))
            };

        Self {
            project,
            element_focus: ElementFocus::Collections,
            current_request_idx: request_idx,
            responses: Responses::new(),
            __send_event,
        }
    }

    pub fn reader(&self) -> PaneStateReader<'_> {
        PaneStateReader::new(&self)
    }

    pub fn project_name(&self) -> &str {
        self.project.name()
    }

    fn set_request_idx(&mut self, idx_opt: Option<(usize, usize)>) {
        match idx_opt {
            Some((idx, sub_idx)) => {
                let exist_request = self
                    .project
                    .collections()
                    .get(idx)
                    .and_then(|coll| coll.requests().get(sub_idx))
                    .is_some();

                if exist_request {
                    self.current_request_idx = Some((idx, sub_idx));
                }
            }
            None => {
                if self.project.collections().is_empty()
                    || self.project.collections()[0].requests().is_empty()
                {
                    self.current_request_idx = None;
                }
            }
        }
    }

    pub fn delete_collection(&mut self, idx: usize) -> CollectionsModel {
        self.project.remove_collection(idx)
    }

    pub fn delete_request(&mut self, idx: (usize, usize)) -> Option<RequestModel> {
        self.project.remove_request(idx)
    }

    pub fn add_collection(&mut self, collection: CollectionsModel) {
        self.project.add_collection(collection);
    }

    pub fn add_request(&mut self, coll_idx: usize, req: RequestModel) {
        self.project.add_request_by_i(coll_idx, req);
    }

    pub fn edit_collection_name(&mut self, idx: usize, name: String) {
        if let Some(collection) = self.project.collection_by_idx_mut(idx) {
            collection.set_name(name);
        }
    }

    pub fn edit_request<F: FnMut(&mut RequestModel)>(&mut self, idx: (usize, usize), mut cb: F) {
        if let Some(request) = self.project.request_mut_by_idx(idx) {
            cb(request);
        }
    }

    pub fn focus_element(&mut self, element_focus: ElementFocus) {
        self.element_focus = element_focus;
    }

    pub fn focus(&self) -> &ElementFocus {
        &self.element_focus
    }

    pub fn is_focused(&self, element_focus: ElementFocus) -> bool {
        self.element_focus == element_focus
    }

    pub fn send_redraw(&self) {
        let _ = self.__send_event.send(Event::Draw);
    }
}

pub struct PaneStateReader<'a> {
    state: &'a PaneState,
}

impl<'a> PaneStateReader<'a> {
    fn new(state: &'a PaneState) -> Self {
        Self { state }
    }

    pub fn current_request_idx(&self) -> Option<(usize, usize)> {
        self.state.current_request_idx
    }

    pub fn current_request(&self) -> Option<&RequestModel> {
        match self.state.current_request_idx {
            Some(idx) => self.state.project.request_by_idx(idx),
            None => None,
        }
    }

    pub fn collections(&self) -> &[CollectionsModel] {
        self.state.project.collections()
    }

    pub fn is_sending_request(&self, id: SendRequestId) -> bool {
        match self.state.responses.get(&id) {
            Some(send_req) => match &*send_req.read().unwrap() {
                crate::store::models::SendRequest::Pending => true,
                crate::store::models::SendRequest::Finish(_) => false,
            },
            None => false,
        }
    }
}
