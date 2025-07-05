use crate::store::models::{CollectionsModel, ProjectModel, RequestModel};

use super::focus::{ElementFocus, OverlayFocus};

pub mod history;
pub mod mutations;

pub struct PaneState {
    project: ProjectModel,
    element_focus: ElementFocus,
    overlay_focus: Option<OverlayFocus>,
    current_request_idx: Option<(usize, usize)>,
}

impl<'a> PaneState {
    pub fn new(project: ProjectModel) -> Self {
        let request_idx =
            if project.collections().is_empty() || project.collections()[0].requests().is_empty() {
                None
            } else {
                Some((0, 0))
            };

        Self {
            project,
            element_focus: ElementFocus::Collections,
            overlay_focus: None,
            current_request_idx: request_idx,
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

    pub fn focus_overlay(&mut self, overlay_focus: OverlayFocus) {
        self.overlay_focus = Some(overlay_focus);
    }

    pub fn hidden_overlay(&mut self) {
        self.overlay_focus = None;
    }

    pub fn is_focused(&self, element_focus: ElementFocus) -> bool {
        self.element_focus == element_focus
    }
}

pub struct PaneStateReader<'a> {
    state: &'a PaneState,
}

impl<'a> PaneStateReader<'a> {
    fn new(state: &'a PaneState) -> Self {
        Self { state }
    }

    pub fn project_name(&self) -> &str {
        self.state.project.name()
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
}
