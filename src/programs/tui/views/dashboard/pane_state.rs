use crate::store::models::ProjectModel;

use super::focus::{ElementFocus, OverlayFocus};

pub struct PaneState {
    project: ProjectModel,
    element_focus: ElementFocus,
    overlay_focus: Option<OverlayFocus>,
}

impl PaneState {
    pub fn new(project: ProjectModel) -> Self {
        Self {
            project,
            element_focus: ElementFocus::Collections,
            overlay_focus: None,
        }
    }

    pub fn project_ref(&self) -> &ProjectModel {
        &self.project
    }

    pub fn project_mut(&mut self) -> &mut ProjectModel {
        &mut self.project
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

    pub fn next_focus_element(&mut self) {
        self.element_focus = match self.element_focus {
            ElementFocus::Collections => ElementFocus::ResponseViewer,
            ElementFocus::MethodUrlBar => ElementFocus::RequestBuilder,
            ElementFocus::RequestBuilder => ElementFocus::Collections,
            ElementFocus::ResponseViewer => ElementFocus::RequestBuilder,
        };
    }

    pub fn prev_focus_element(&mut self) {
        self.element_focus = match self.element_focus {
            ElementFocus::Collections => ElementFocus::ResponseViewer,
            ElementFocus::MethodUrlBar => ElementFocus::Collections,
            ElementFocus::RequestBuilder => ElementFocus::Collections,
            ElementFocus::ResponseViewer => ElementFocus::RequestBuilder,
        };
    }

    pub fn is_element_focus(&self, _element_focus: ElementFocus) -> bool {
        self.element_focus == _element_focus
    }
}
