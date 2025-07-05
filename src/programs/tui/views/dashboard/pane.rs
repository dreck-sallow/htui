use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    Frame,
};

use crate::{
    programs::tui::element_view::{Drawable, ElementView, Interactive, Painter},
    store::models::ProjectModel,
};

use super::{
    collections::CollectionsComponent,
    method_url_bar::MethodUrlBarComponent,
    pane_state::{history::MutationsHistory, PaneState},
    placeholder::PlaceholderView,
    request_builder::RequestEditorComponent,
    response_viewer::ResponseViewerView,
};

pub struct PaneView<'a> {
    render_area: Rect,
    pane_state: PaneState,
    mutations_history: MutationsHistory<'a>,
    // request_editor_view: RequestEditorView,
    response_viewer_view: ResponseViewerView,
    placeholder_view: PlaceholderView,

    collections_component: CollectionsComponent,
    method_url_bar_component: MethodUrlBarComponent,
    request_editor_component: RequestEditorComponent,
}

impl PaneView<'_> {
    pub fn new(project: ProjectModel) -> Self {
        Self {
            render_area: Rect::default(),
            collections_component: CollectionsComponent::new(&project),
            method_url_bar_component: MethodUrlBarComponent::new(),
            request_editor_component: RequestEditorComponent::new(),

            pane_state: PaneState::new(project),
            mutations_history: MutationsHistory::new(),
            // request_editor_view,
            response_viewer_view: ResponseViewerView::new(),
            placeholder_view: PlaceholderView::new(),
        }
    }

    pub fn project_name(&self) -> &str {
        self.pane_state.project_name()
    }
}

impl<'a> ElementView<'a> for PaneView<'_> {
    type State = ();
    type Collector = ();

    fn draw(&self, frame: &mut Frame, _state: &Self::State) {
        let mut painter = Painter::new();
        self.collections_component
            .draw(&mut painter, &self.pane_state);

        self.method_url_bar_component
            .draw(&mut painter, &self.pane_state);

        self.request_editor_component
            .draw(&mut painter, &self.pane_state);

        painter.draw(frame);

        // let state_reader = self.pane_state.reader();
        // self.collections_view.draw(frame, &state_reader);

        // if state_reader.current_request_idx().is_some() {
        //     self.method_url_bar_view.draw(frame, &state_reader);

        //     self.request_editor_view.draw(frame, &state_reader);

        //     self.response_viewer_view.draw(frame, &state_reader);
        // } else {
        //     self.placeholder_view.draw(frame, &state_reader);
        // }

        // if let Some(overlay_focus) = self.global_pane_state.overlay() {
        //     match overlay_focus {
        //         super::focus::OverlayFocus::UpsertItem => {
        //             self.upsert_item_view.draw(frame, &self.global_pane_state)
        //         }
        //         super::focus::OverlayFocus::MethodSelector => self
        //             .method_selector_view
        //             .draw(frame, &self.global_pane_state),
        //         super::focus::OverlayFocus::BodySelector => {
        //             self.body_selector_view.draw(frame, &self.global_pane_state)
        //         }
        //     }
        // }
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
        self.collections_component.set_render_area(collections_area);
        self.method_url_bar_component
            .set_render_area(content_areas[0]);
        self.request_editor_component
            .set_render_area(content_areas[1]);

        self.response_viewer_view.set_area(content_areas[2]);
        self.placeholder_view.set_area(placeholder_area);
        self.render_area = area;
    }

    fn on_key(&mut self, key: KeyEvent, _state: &mut Self::State) {
        let mut mutations_collector = self.mutations_history.collector();

        match self.pane_state.focus() {
            super::focus::ElementFocus::Collections => {
                self.collections_component
                    .on_key(key, &mut mutations_collector, &self.pane_state);
            }
            super::focus::ElementFocus::MethodUrlBar => {
                self.method_url_bar_component.on_key(
                    key,
                    &mut mutations_collector,
                    &self.pane_state,
                );
            }
            super::focus::ElementFocus::RequestBuilder => {
                self.request_editor_component.on_key(
                    key,
                    &mut mutations_collector,
                    &self.pane_state,
                );
            }
            super::focus::ElementFocus::ResponseViewer => {}
        }

        if mutations_collector.has_content() {
            self.mutations_history
                .apply_from_collector(mutations_collector, &mut self.pane_state);

            self.collections_component.on_change_state(&self.pane_state);
            self.method_url_bar_component
                .on_change_state(&self.pane_state);

            self.request_editor_component
                .on_change_state(&self.pane_state);
        }

        // if let Some(overlay_focus) = self.global_pane_state.overlay() {
        //     match overlay_focus {
        //         super::focus::OverlayFocus::UpsertItem => self
        //             .upsert_item_view
        //             .on_key(key, &mut self.global_pane_state),
        //         super::focus::OverlayFocus::MethodSelector => {
        //             self.method_selector_view
        //                 .on_key(key, &mut self.global_pane_state);
        //         }
        //         super::focus::OverlayFocus::BodySelector => self
        //             .body_selector_view
        //             .on_key(key, &mut self.global_pane_state),
        //     }
        // } else {
        //     match self.global_pane_state.element_focus() {
        //         super::focus::ElementFocus::Collections => self
        //             .collections_view
        //             .on_key(key, &mut self.global_pane_state),
        //         super::focus::ElementFocus::MethodUrlBar => self
        //             .method_url_bar_view
        //             .on_key(key, &mut self.global_pane_state),
        //         super::focus::ElementFocus::RequestBuilder => {
        //             self.request_editor_view
        //                 .on_key(key, &mut self.global_pane_state);
        //         }
        //         super::focus::ElementFocus::ResponseViewer => self
        //             .response_viewer_view
        //             .on_key(key, &mut self.global_pane_state),
        //     }

        //     // TODO: when open a overlay, react to the previous changes
        //     // if let Some(overlay_focus) = self.global_pane_state.overlay() {
        //     //     match overlay_focus {
        //     //         super::focus::OverlayFocus::UpsertItem => {
        //     //             self.upsert_item_view.set_inner(&self.global_pane_state)
        //     //         }
        //     //         super::focus::OverlayFocus::MethodSelector => {
        //     //             // self.method_selector_view.draw(frame)
        //     //         }
        //     //         super::focus::OverlayFocus::BodySelector => {}
        //     //     }
        //     // }

        //     // FIXME: call on_change_state only when the global_pane_state was changed
        //     self.method_url_bar_view.on_change_state(&reader);

        //     self.request_editor_view.on_change_state(&reader);
        // }
    }

    fn on_change_state(&mut self, _state: &Self::State) {}
}
