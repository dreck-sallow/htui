use ratatui::{
    layout::{Constraint, Layout, Rect},
    text::Span,
};

use crate::programs::tui::element_view::ElementView;

use super::global_pane_state::GlobalPaneState;

pub struct PlaceholderView {
    render_area: Rect,
}

impl PlaceholderView {
    pub fn new() -> Self {
        Self {
            render_area: Rect::default(),
        }
    }
}

impl ElementView<'_> for PlaceholderView {
    type State = GlobalPaneState;
    type Collector = ();

    fn draw(&self, frame: &mut ratatui::Frame, _state: &Self::State) {
        let [x_area] = Layout::horizontal([Constraint::Percentage(40)])
            .flex(ratatui::layout::Flex::Center)
            .areas(self.render_area);

        let [header_area, desc_area, shortcuts_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .spacing(1)
        .flex(ratatui::layout::Flex::Center)
        .areas(x_area);

        frame.render_widget(Span::from("HTUI - Terminal HTTP Client"), header_area);
        frame.render_widget(Span::from("No request selected."), desc_area);
        frame.render_widget(
            Span::from("[n] New Request - [c] New Collection"),
            shortcuts_area,
        );
    }

    fn set_area(&mut self, area: Rect) {
        self.render_area = area;
    }
}
