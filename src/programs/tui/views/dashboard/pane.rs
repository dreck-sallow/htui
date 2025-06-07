use std::collections::VecDeque;

use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    Frame,
};

use crate::store::models::ProjectModel;

use super::{
    action::Action, collections::CollectionsView, method_selector::MethodSelectorView,
    pane_state::PaneState, request_builder::RequestBuilderView, upsert_item::UpsertItemView,
};

pub struct PaneView {
    state: PaneState,
    collections_view: CollectionsView,
    request_builder_view: RequestBuilderView,
    upsert_item_view: UpsertItemView,
    method_selector_view: MethodSelectorView,
}

impl PaneView {
    pub fn new(project: ProjectModel) -> Self {
        let mut collections_view = CollectionsView::new();

        for collection in project.collections() {
            collections_view.insert_collection(collection);
        }

        Self {
            state: PaneState::new(project),
            collections_view,
            upsert_item_view: UpsertItemView::new(),
            request_builder_view: RequestBuilderView::new(),
            method_selector_view: MethodSelectorView::new(),
        }
    }

    pub fn project(&self) -> &ProjectModel {
        &self.state.project_ref()
    }

    fn propagate_actions(&mut self, mut actions_acc: VecDeque<Action>) {
        while let Some(action) = actions_acc.pop_front() {
            self.collections_view
                .handle_action(action.clone(), &mut self.state);
            self.upsert_item_view.handle_action(action.clone());
            self.request_builder_view.handle_action(action.clone());
            self.method_selector_view.handle_action(action.clone());
        }
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let areas = Layout::horizontal([Constraint::Percentage(25), Constraint::Fill(1)])
            .spacing(1)
            .split(area);

        self.collections_view.render(
            self.project(),
            frame,
            areas[0],
            self.state
                .is_element_focus(super::focus::ElementFocus::Collections),
        );

        self.request_builder_view.draw(
            frame,
            areas[1],
            self.state
                .is_element_focus(super::focus::ElementFocus::RequestBuilder),
        );

        if let Some(overlay_focus) = self.state.overlay_focus() {
            match overlay_focus {
                super::focus::OverlayFocus::UpsertItem => self.upsert_item_view.draw(frame),
                super::focus::OverlayFocus::MethodSelector => self.method_selector_view.draw(frame),
            }
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        let mut acc_actions = Vec::new();

        if let Some(overlay_focus) = self.state.overlay_focus() {
            match overlay_focus {
                super::focus::OverlayFocus::UpsertItem => {
                    self.upsert_item_view
                        .handle_key(key, &mut self.state, &mut acc_actions)
                }
                super::focus::OverlayFocus::MethodSelector => {
                    self.method_selector_view
                        .handle_key(key, &mut self.state, &mut acc_actions);
                }
            }
        } else {
            match self.state.element_focus() {
                super::focus::ElementFocus::Collections => {
                    self.collections_view
                        .handle_key(key, &mut self.state, &mut acc_actions)
                }
                super::focus::ElementFocus::RequestBuilder => {
                    self.request_builder_view
                        .handle_key(key, &mut self.state, &mut acc_actions)
                }
                super::focus::ElementFocus::ResponseViewer => todo!(),
            }
        }

        self.propagate_actions(acc_actions.into());
    }
}
