use std::collections::HashSet;

use crate::store::models::{CollectionsModel, ProjectModel, RequestModel};

use super::{
    collections::{CollectionsState, Idx},
    focus::{ElementFocus, OverlayFocus},
    method_selector::MethodSelectorState,
    request_builder::request_builder_state::RequestBuilderState,
    response_viewer::response_viewer_state::ResponseViewerState,
    upsert_item::{upsert_item_state::UpsertItemState, UpsertMethod},
};

#[derive(Clone, Copy)]
pub enum CollectionChange {
    CloseCollection,
    OpenCollection,
    NextItem,
    PrevItem,
    Select,
}

pub struct FocusState {
    element_focus: ElementFocus,
    overlay_focus: Option<OverlayFocus>,
}

impl FocusState {
    pub fn new(element_focus: ElementFocus) -> Self {
        Self {
            element_focus,
            overlay_focus: None,
        }
    }

    pub fn element_focus(&self) -> &ElementFocus {
        &self.element_focus
    }

    pub fn overlay_focus(&self) -> Option<&OverlayFocus> {
        self.overlay_focus.as_ref()
    }

    pub fn focus_overlay(&mut self, overlay_focus: OverlayFocus) {
        self.overlay_focus = Some(overlay_focus);
    }

    pub fn hidden_overlay(&mut self) {
        self.overlay_focus = None;
    }

    pub fn focus_element(&mut self, element_focus: ElementFocus) {
        self.element_focus = element_focus;
    }

    pub fn is_element_focus(&self, element_focus: ElementFocus) -> bool {
        self.element_focus == element_focus
    }
}

pub struct GlobalPaneState {
    project: ProjectModel,
    pub current_request_idx: Option<(usize, usize)>,
    focus_state: FocusState,
    collections_state: CollectionsState<String>,
    request_builder_state: RequestBuilderState,
    response_viewer_state: ResponseViewerState,
    upsert_item_state: UpsertItemState,
    method_selector_state: MethodSelectorState,
}

impl GlobalPaneState {
    pub fn new(project: ProjectModel) -> Self {
        let mut collections_state = CollectionsState::new();

        for collection in project.collections() {
            let children = collection
                .requests()
                .iter()
                .map(|req| req.id().to_string())
                .collect();

            collections_state.add_collection((collection.id().to_string(), children));
        }

        let current_request_idx = match collections_state.idx() {
            Idx::Child(i, sub_i) => Some((i, sub_i)),
            _ => None,
        };

        Self {
            project: project,
            current_request_idx,
            focus_state: FocusState::new(ElementFocus::Collections),
            collections_state,
            request_builder_state: RequestBuilderState::new(),
            upsert_item_state: UpsertItemState::new(),
            response_viewer_state: ResponseViewerState::new(),
            method_selector_state: MethodSelectorState::new(),
        }
    }

    pub fn project_ref(&self) -> &ProjectModel {
        &self.project
    }

    pub fn project_collections(&self) -> &[CollectionsModel] {
        self.project.collections()
    }

    pub fn project_name(&self) -> &str {
        self.project.name()
    }

    pub fn is_focus(&self, element_focus: ElementFocus) -> bool {
        self.focus_state.is_element_focus(element_focus)
    }

    pub fn overlay(&self) -> Option<&OverlayFocus> {
        self.focus_state.overlay_focus()
    }

    pub fn element_focus(&self) -> &ElementFocus {
        self.focus_state.element_focus()
    }

    pub fn set_focus(&mut self, element_focus: ElementFocus) {
        self.focus_state.focus_element(element_focus);
    }

    pub fn set_overlay(&mut self, overlay: OverlayFocus) {
        self.focus_state.focus_overlay(overlay);
    }

    pub fn hidden_overlay(&mut self) {
        self.focus_state.hidden_overlay();
    }

    pub fn current_request(&self) -> Option<&RequestModel> {
        self.current_request_idx
            .and_then(|i| self.project.request_by_idx(i))
    }

    pub fn collections_raw_data(&self) -> (HashSet<usize>, Idx) {
        (
            self.collections_state.openeds(),
            self.collections_state.idx(),
        )
    }

    pub fn collection_change(&mut self, change: CollectionChange) {
        match change {
            CollectionChange::CloseCollection => self.collections_state.close_collection(true),
            CollectionChange::OpenCollection => self.collections_state.open_collection(true),
            CollectionChange::NextItem => self.collections_state.next(),
            CollectionChange::PrevItem => self.collections_state.prev(),
            CollectionChange::Select => {
                // FIXME: add to other mutate indices
                self.current_request_idx = match self.collections_state.idx() {
                    Idx::Child(i, sub_i) => Some((i, sub_i)),
                    _ => None,
                };
            }
        }
    }

    pub fn add_collection(&mut self, collection: CollectionsModel) {
        let children = collection
            .requests()
            .iter()
            .map(|req| req.id().to_string())
            .collect();

        self.collections_state
            .add_collection((collection.id().to_string(), children));
        self.project.add_collection(collection);
    }

    pub fn edit_item_name(&mut self, name: String) {
        match self.collections_state.idx() {
            Idx::None => {}
            Idx::Parent(i) => {
                if let Some(coll) = self.project.collection_by_idx_mut(i) {
                    coll.set_name(name);
                };
            }
            Idx::Child(i, sub_i) => {
                if let Some(req) = self
                    .project
                    .collection_by_idx_mut(i)
                    .and_then(|coll| coll.get_request_mut(sub_i))
                {
                    req.set_name(name);
                };
            }
        }
    }

    pub fn add_collection_request(&mut self, request: RequestModel) {
        match self.collections_state.idx() {
            Idx::None => {}
            Idx::Parent(i) | Idx::Child(i, _) => {
                self.collections_state
                    .add_request_on_current(request.id().to_string());
                self.project.add_request_by_i(i, request);
            }
        }
    }

    pub fn delete_collection_item(&mut self, (i, sub_i_opt): (usize, Option<usize>)) {
        match sub_i_opt {
            Some(sub_i) => {
                self.project.remove_request((i, sub_i));
                self.collections_state.delete();
            }
            None => {
                self.project.remove_collection(i);
                self.collections_state.delete();
            }
        }
    }

    pub fn change_method_from_state(&mut self) {
        if let Some(idx) = self.current_request_idx {
            if let Some(req) = self.project.request_mut_by_idx(idx) {
                req.set_method(self.method_selector_state.inner());
            }
        }
    }

    pub fn change_url_from_state(&mut self, url: String) {
        if let Some(idx) = self.current_request_idx {
            if let Some(req) = self.project.request_mut_by_idx(idx) {
                req.set_url(url);
            }
        }
    }

    pub fn set_upsert_form(&mut self, method: UpsertMethod) {
        self.upsert_item_state.set_method(method);
    }

    pub fn upsert_method(&self) -> UpsertMethod {
        self.upsert_item_state.method()
    }

    pub fn builder_state_ref(&self) -> &RequestBuilderState {
        &self.request_builder_state
    }

    pub fn builder_state_mut(&mut self) -> &mut RequestBuilderState {
        &mut self.request_builder_state
    }

    pub fn response_state_ref(&self) -> &ResponseViewerState {
        &self.response_viewer_state
    }

    pub fn response_state_mut(&mut self) -> &mut ResponseViewerState {
        &mut self.response_viewer_state
    }

    pub fn method_selector_state_ref(&self) -> &MethodSelectorState {
        &self.method_selector_state
    }

    pub fn method_selector_state_mut(&mut self) -> &mut MethodSelectorState {
        &mut self.method_selector_state
    }
}
