use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    Frame,
};

use crate::{
    programs::tui::element_view::{Drawable, Interactive, Painter},
    store::models::ProjectModel,
};

use super::{
    collections::CollectionsComponent,
    method_url_bar::MethodUrlBarComponent,
    pane_state::{history::MutationsHistory, PaneState},
    placeholder::PlaceholderView,
    request_builder::RequestEditorComponent,
    response_viewer::ResponseViewerComponent,
};

pub struct PaneView<'a> {
    render_area: Rect,
    pane_state: PaneState,
    mutations_history: MutationsHistory<'a>,
    placeholder_view: PlaceholderView,

    collections_component: CollectionsComponent,
    method_url_bar_component: MethodUrlBarComponent,
    request_editor_component: RequestEditorComponent,
    response_viewer_component: ResponseViewerComponent,
}

impl PaneView<'_> {
    pub fn new(project: ProjectModel) -> Self {
        Self {
            render_area: Rect::default(),
            collections_component: CollectionsComponent::new(&project),
            method_url_bar_component: MethodUrlBarComponent::new(),
            request_editor_component: RequestEditorComponent::new(),
            response_viewer_component: ResponseViewerComponent::new(),

            pane_state: PaneState::new(project),
            mutations_history: MutationsHistory::new(),
            placeholder_view: PlaceholderView::new(),
        }
    }

    pub fn project_name(&self) -> &str {
        self.pane_state.project_name()
    }
}

impl PaneView<'_> {
    pub fn draw(&self, frame: &mut Frame) {
        let mut painter = Painter::new();
        self.collections_component
            .draw(&mut painter, &self.pane_state);

        if self.pane_state.reader().current_request_idx().is_some() {
            self.method_url_bar_component
                .draw(&mut painter, &self.pane_state);

            self.request_editor_component
                .draw(&mut painter, &self.pane_state);

            self.response_viewer_component
                .draw(&mut painter, &self.pane_state);
        } else {
            // TODO: update to new rendering flow
            self.placeholder_view.draw(frame);
        }

        painter.draw(frame);
    }

    pub fn set_area(&mut self, area: Rect) {
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

        self.response_viewer_component
            .set_render_area(content_areas[2]);
        self.placeholder_view.set_area(placeholder_area);
        self.render_area = area;
    }

    pub fn on_key(&mut self, key: KeyEvent) {
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
            super::focus::ElementFocus::ResponseViewer => {
                self.response_viewer_component.on_key(
                    key,
                    &mut mutations_collector,
                    &self.pane_state,
                );
            }
        }

        if mutations_collector.has_content() {
            self.mutations_history
                .apply_from_collector(mutations_collector, &mut self.pane_state);

            self.collections_component.on_change_state(&self.pane_state);
            self.method_url_bar_component
                .on_change_state(&self.pane_state);

            self.request_editor_component
                .on_change_state(&self.pane_state);

            self.response_viewer_component
                .on_change_state(&self.pane_state);
        }
    }
}
