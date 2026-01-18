use ratatui::{layout::Rect, style::Style, text::Line, Frame};

pub struct PlaceholderLine {
    text: String,
    style: Style,
}

impl PlaceholderLine {
    pub fn new<S: Into<String>>(s: S) -> Self {
        Self {
            text: s.into(),
            style: Style::default(),
        }
    }

    pub fn with_style(mut self, s: Style) -> Self {
        self.style = s;
        self
    }
}

impl PlaceholderLine {
    pub fn draw(&self, area: Rect, frame: &mut Frame) {
        let line = Line::raw(&self.text).style(self.style);

        frame.render_widget(line, area);
    }
}
