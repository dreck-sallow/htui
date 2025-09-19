use ratatui::{
    layout::{Constraint, Layout, Rect},
    text::Span,
};

pub struct PlaceholderView {
    render_area: Rect,
}

impl PlaceholderView {
    pub fn new() -> Self {
        Self {
            render_area: Rect::default(),
        }
    }

    pub fn draw(&self, frame: &mut ratatui::Frame) {
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
            Span::from("[r] New Request - [c] New Collection"),
            shortcuts_area,
        );
    }

    pub fn set_render_area(&mut self, area: Rect) {
        self.render_area = area;
    }
}
