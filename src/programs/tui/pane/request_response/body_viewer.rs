use crate::programs::tui::pane::text_editor::TextEditor;
use ratatui::{layout::Rect, style::Stylize, text::Span, widgets::Widget};

use super::binary_viewer::BinaryViewer;

pub enum BodyContentView {
    Text(TextEditor),
    Binary(BinaryViewer),
    Empty,
}

impl Widget for &BodyContentView {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        match self {
            BodyContentView::Text(text_editor) => text_editor.render(area, buf),
            BodyContentView::Binary(binary_viewer) => binary_viewer.render(area, buf),
            BodyContentView::Empty => {
                if area.is_empty() {
                    return;
                }

                buf.set_span(
                    area.left(),
                    area.top(),
                    &Span::from("No body content").italic(),
                    area.width,
                );
            }
        }
    }
}
