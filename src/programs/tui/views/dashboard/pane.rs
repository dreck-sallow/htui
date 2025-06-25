use std::collections::VecDeque;

use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    Frame,
};

use crate::store::models::ProjectModel;

use super::{
    action::Action, collections::CollectionsView, global_pane_state::GlobalPaneState,
    method_selector::MethodSelectorView, request_builder::RequestBuilderView,
    upsert_item::UpsertItemView,
};

pub struct PaneView {
    global_pane_state: GlobalPaneState,
    collections_view: CollectionsView,
    request_builder_view: RequestBuilderView,
    upsert_item_view: UpsertItemView,
    method_selector_view: MethodSelectorView,
}

impl PaneView {
    pub fn new(project: ProjectModel) -> Self {
        Self {
            global_pane_state: GlobalPaneState::new(project),
            collections_view: CollectionsView::new(),
            upsert_item_view: UpsertItemView::new(),
            request_builder_view: RequestBuilderView::new(),
            method_selector_view: MethodSelectorView::new(),
        }
    }

    pub fn project_name(&self) -> &str {
        self.global_pane_state.project_name()
    }

    fn propagate_actions(&mut self, mut actions_acc: VecDeque<Action>) {
        while let Some(action) = actions_acc.pop_front() {
            // self.collections_view
            // .handle_action(action.clone(), &mut self.state);
            self.upsert_item_view.handle_action(action.clone());
            self.request_builder_view.handle_action(action.clone());
            self.method_selector_view.handle_action(action.clone());
        }
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let areas = Layout::horizontal([Constraint::Percentage(25), Constraint::Fill(1)])
            .spacing(1)
            .split(area);

        self.collections_view
            .render(frame, areas[0], &self.global_pane_state);

        // self.request_builder_view.draw(
        //     frame,
        //     areas[1],
        //     self.state
        //         .is_element_focus(super::focus::ElementFocus::RequestBuilder),
        // );

        if let Some(overlay_focus) = self.global_pane_state.overlay() {
            match overlay_focus {
                super::focus::OverlayFocus::UpsertItem => self.upsert_item_view.draw(frame),
                super::focus::OverlayFocus::MethodSelector => self.method_selector_view.draw(frame),
            }
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        let mut acc_actions = Vec::new();

        if let Some(overlay_focus) = self.global_pane_state.overlay() {
            match overlay_focus {
                super::focus::OverlayFocus::UpsertItem => self
                    .upsert_item_view
                    .handle_key(key, &mut self.global_pane_state),
                super::focus::OverlayFocus::MethodSelector => {
                    // self.method_selector_view
                    //     .handle_key(key, &mut self.state, &mut acc_actions);
                }
            }
        } else {
            match self.global_pane_state.element_focus() {
                super::focus::ElementFocus::Collections => self
                    .collections_view
                    .handle_key(key, &mut self.global_pane_state),
                super::focus::ElementFocus::RequestBuilder => {
                    // self.request_builder_view
                    //     .handle_key(key, &mut self.state, &mut acc_actions)
                }
                super::focus::ElementFocus::ResponseViewer => todo!(),
            }

            // TODO: when open a overlay, react to the previous changes
            if let Some(overlay_focus) = self.global_pane_state.overlay() {
                match overlay_focus {
                    super::focus::OverlayFocus::UpsertItem => {
                        self.upsert_item_view.set_inner(&self.global_pane_state)
                    }
                    super::focus::OverlayFocus::MethodSelector => {
                        // self.method_selector_view.draw(frame)
                    }
                }
            }
        }

        self.propagate_actions(acc_actions.into());
    }
}
