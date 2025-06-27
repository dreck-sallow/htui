use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    Frame,
};

use crate::{programs::tui::element_view::ElementView, store::models::ProjectModel};

use super::{
    collections::CollectionsView, global_pane_state::GlobalPaneState,
    method_selector::MethodSelectorView, request_builder::RequestBuilderView,
    response_viewer::ResponseViewerView, upsert_item::UpsertItemView,
};

pub struct PaneView {
    render_area: Rect,
    global_pane_state: GlobalPaneState,
    collections_view: CollectionsView,
    request_builder_view: RequestBuilderView,
    response_viewer_view: ResponseViewerView,
    upsert_item_view: UpsertItemView,
    method_selector_view: MethodSelectorView,
}

impl PaneView {
    pub fn new(project: ProjectModel) -> Self {
        Self {
            render_area: Rect::default(),
            global_pane_state: GlobalPaneState::new(project),
            collections_view: CollectionsView::new(),
            upsert_item_view: UpsertItemView::new(),
            request_builder_view: RequestBuilderView::new(),
            response_viewer_view: ResponseViewerView::new(),
            method_selector_view: MethodSelectorView::new(),
        }
    }

    pub fn project_name(&self) -> &str {
        self.global_pane_state.project_name()
    }

    // pub fn draw(&mut self, frame: &mut Frame, area: Rect) {
    //     let [collections_area, request_area, response_area] = {
    //         let [collections_area, content_area] =
    //             Layout::horizontal([Constraint::Percentage(25), Constraint::Fill(1)])
    //                 .spacing(1)
    //                 .areas(area);

    //         let [request_area, response_area] =
    //             Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
    //                 .areas(content_area);

    //         [collections_area, request_area, response_area]
    //     };

    //     self.collections_view
    //         .render(frame, collections_area, &self.global_pane_state);

    //     self.request_builder_view
    //         .draw(frame, request_area, &self.global_pane_state);

    //     self.response_viewer_view
    //         .draw(frame, response_area, &self.global_pane_state);

    //     if let Some(overlay_focus) = self.global_pane_state.overlay() {
    //         match overlay_focus {
    //             super::focus::OverlayFocus::UpsertItem => self.upsert_item_view.draw(frame),
    //             super::focus::OverlayFocus::MethodSelector => self.method_selector_view.draw(frame),
    //         }
    //     }
    // }

    // pub fn handle_key_event(&mut self, key: KeyEvent) {
    //     if let Some(overlay_focus) = self.global_pane_state.overlay() {
    //         match overlay_focus {
    //             super::focus::OverlayFocus::UpsertItem => self
    //                 .upsert_item_view
    //                 .handle_key(key, &mut self.global_pane_state),
    //             super::focus::OverlayFocus::MethodSelector => {
    //                 // self.method_selector_view
    //                 //     .handle_key(key, &mut self.state, &mut acc_actions);
    //             }
    //         }
    //     } else {
    //         match self.global_pane_state.element_focus() {
    //             super::focus::ElementFocus::Collections => self
    //                 .collections_view
    //                 .handle_key(key, &mut self.global_pane_state),
    //             super::focus::ElementFocus::RequestBuilder => self
    //                 .request_builder_view
    //                 .handle_key(key, &mut self.global_pane_state),
    //             super::focus::ElementFocus::ResponseViewer => self
    //                 .response_viewer_view
    //                 .handle_key(key, &mut self.global_pane_state),
    //         }

    //         // TODO: when open a overlay, react to the previous changes
    //         if let Some(overlay_focus) = self.global_pane_state.overlay() {
    //             match overlay_focus {
    //                 super::focus::OverlayFocus::UpsertItem => {
    //                     self.upsert_item_view.set_inner(&self.global_pane_state)
    //                 }
    //                 super::focus::OverlayFocus::MethodSelector => {
    //                     // self.method_selector_view.draw(frame)
    //                 }
    //             }
    //         }
    //     }
    // }
}

impl ElementView for PaneView {
    type State = ();

    fn draw(&self, frame: &mut Frame, _state: &Self::State) {
        self.collections_view.draw(frame, &self.global_pane_state);

        self.request_builder_view
            .draw(frame, &self.global_pane_state);

        // self.response_viewer_view
        //     .draw(frame, response_area, &self.global_pane_state);

        // if let Some(overlay_focus) = self.global_pane_state.overlay() {
        //     match overlay_focus {
        //         super::focus::OverlayFocus::UpsertItem => self.upsert_item_view.draw(frame),
        //         super::focus::OverlayFocus::MethodSelector => self.method_selector_view.draw(frame),
        //     }
        // }
    }

    fn set_area(&mut self, _area: Rect) {}

    fn on_key(&mut self, key: KeyEvent, _state: &mut Self::State) {
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
                    .on_key(key, &mut self.global_pane_state),
                super::focus::ElementFocus::RequestBuilder => self
                    .request_builder_view
                    .on_key(key, &mut self.global_pane_state),
                super::focus::ElementFocus::ResponseViewer => self
                    .response_viewer_view
                    .handle_key(key, &mut self.global_pane_state),
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
    }

    fn on_resize(&mut self, area: Rect, _state: &mut Self::State) {
        let [collections_area, request_area, response_area] = {
            let [collections_area, content_area] =
                Layout::horizontal([Constraint::Percentage(25), Constraint::Fill(1)])
                    .spacing(1)
                    .areas(area);

            let [request_area, response_area] =
                Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .areas(content_area);

            [collections_area, request_area, response_area]
        };
        self.collections_view.set_area(collections_area);
        self.request_builder_view
            .on_resize(request_area, &mut self.global_pane_state);
        self.render_area = area;
    }

    fn on_change_state(&mut self, _state: &Self::State) {}
}
