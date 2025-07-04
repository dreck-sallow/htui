use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    Frame,
};

use crate::{programs::tui::element_view::ElementView, store::models::ProjectModel};

use super::{
    body_type_selector::BodyTypeSelectorView, collections::CollectionsView,
    global_pane_state::GlobalPaneState, method_selector::MethodSelectorView,
    method_url_bar::MethodUrlBarView, placeholder::PlaceholderView,
    request_editor::RequestEditorView, response_viewer::ResponseViewerView,
    upsert_item::UpsertItemView,
};

pub struct PaneView {
    render_area: Rect,
    global_pane_state: GlobalPaneState,

    collections_view: CollectionsView,
    method_url_bar_view: MethodUrlBarView,
    request_editor_view: RequestEditorView,
    response_viewer_view: ResponseViewerView,
    // response_viewer_editor: HttpPayloadEditorView,
    upsert_item_view: UpsertItemView,
    method_selector_view: MethodSelectorView,
    body_selector_view: BodyTypeSelectorView,
    placeholder_view: PlaceholderView,
}

impl PaneView {
    pub fn new(project: ProjectModel) -> Self {
        let (method_url_bar_view, method_selector_view) = MethodUrlBarView::new_with_dropdown();
        let (request_editor_view, body_selector_view) = RequestEditorView::new_with_dropdown();

        Self {
            render_area: Rect::default(),
            global_pane_state: GlobalPaneState::new(project),
            collections_view: CollectionsView::new(),
            upsert_item_view: UpsertItemView::new(),
            method_url_bar_view,
            request_editor_view,
            response_viewer_view: ResponseViewerView::new(),
            method_selector_view,
            body_selector_view,
            placeholder_view: PlaceholderView::new(),
        }
    }

    pub fn project_name(&self) -> &str {
        self.global_pane_state.project_name()
    }
}

impl ElementView for PaneView {
    type State = ();

    fn draw(&self, frame: &mut Frame, _state: &Self::State) {
        self.collections_view.draw(frame, &self.global_pane_state);

        if self.global_pane_state.current_request_idx.is_some() {
            self.method_url_bar_view
                .draw(frame, &self.global_pane_state);

            self.request_editor_view
                .draw(frame, &self.global_pane_state);

            self.response_viewer_view
                .draw(frame, &self.global_pane_state);
        } else {
            self.placeholder_view.draw(frame, &self.global_pane_state);
        }

        if let Some(overlay_focus) = self.global_pane_state.overlay() {
            match overlay_focus {
                super::focus::OverlayFocus::UpsertItem => {
                    self.upsert_item_view.draw(frame, &self.global_pane_state)
                }
                super::focus::OverlayFocus::MethodSelector => self
                    .method_selector_view
                    .draw(frame, &self.global_pane_state),
                super::focus::OverlayFocus::BodySelector => {
                    self.body_selector_view.draw(frame, &self.global_pane_state)
                }
            }
        }
    }

    fn set_area(&mut self, area: Rect) {
        let (collections_area, placeholder_area, content_areas) = {
            let [collections_area, content_area] =
                Layout::horizontal([Constraint::Percentage(25), Constraint::Fill(1)])
                    .spacing(1)
                    .areas(area);

            let right_areas = Layout::vertical([
                Constraint::Length(3),
                Constraint::Fill(1),
                Constraint::Fill(1),
            ])
            .split(content_area);

            (collections_area, content_area, right_areas)
        };
        self.collections_view.set_area(collections_area);
        self.method_url_bar_view.set_area(content_areas[0]);
        self.request_editor_view.set_area(content_areas[1]);
        self.response_viewer_view.set_area(content_areas[2]);
        self.placeholder_view.set_area(placeholder_area);
        self.upsert_item_view.set_area(area);
        self.render_area = area;
    }

    fn on_key(&mut self, key: KeyEvent, _state: &mut Self::State) {
        if let Some(overlay_focus) = self.global_pane_state.overlay() {
            match overlay_focus {
                super::focus::OverlayFocus::UpsertItem => self
                    .upsert_item_view
                    .on_key(key, &mut self.global_pane_state),
                super::focus::OverlayFocus::MethodSelector => {
                    self.method_selector_view
                        .on_key(key, &mut self.global_pane_state);
                }
                super::focus::OverlayFocus::BodySelector => self
                    .body_selector_view
                    .on_key(key, &mut self.global_pane_state),
            }
        } else {
            match self.global_pane_state.element_focus() {
                super::focus::ElementFocus::Collections => self
                    .collections_view
                    .on_key(key, &mut self.global_pane_state),
                super::focus::ElementFocus::MethodUrlBar => self
                    .method_url_bar_view
                    .on_key(key, &mut self.global_pane_state),
                super::focus::ElementFocus::RequestBuilder => {
                    self.request_editor_view
                        .on_key(key, &mut self.global_pane_state);
                }
                super::focus::ElementFocus::ResponseViewer => self
                    .response_viewer_view
                    .on_key(key, &mut self.global_pane_state),
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
                    super::focus::OverlayFocus::BodySelector => {}
                }
            }

            // FIXME: call on_change_state only when the global_pane_state was changed
            self.method_url_bar_view
                .on_change_state(&self.global_pane_state);

            self.request_editor_view
                .on_change_state(&self.global_pane_state);
        }
    }

    fn on_change_state(&mut self, _state: &Self::State) {}
}
