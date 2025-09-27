use crate::programs::tui::{
    common::UiComposedElement, config::Config, pane::text_editor::TextEditor,
};
use ratatui::{layout::Rect, style::Stylize, text::Span};

use super::binary_body::BinaryViewer;

pub enum BodyContentView {
    Text { area: Rect, editor: TextEditor },
    Binary(BinaryViewer),
    Empty(BodyEmptyView),
}

impl BodyContentView {
    pub fn empty(area: Rect) -> Self {
        Self::Empty(BodyEmptyView { area })
    }

    pub fn text(editor: TextEditor, area: Rect) -> Self {
        Self::Text { area, editor }
    }
}

impl<'params> UiComposedElement<'params> for BodyContentView {
    type Params = &'params Config;

    fn set_area(&mut self, area: Rect, viewport_area: Rect) {
        match self {
            BodyContentView::Text { .. } => {}
            BodyContentView::Binary(binary_viewer) => {
                binary_viewer.set_area(area, viewport_area);
            }
            BodyContentView::Empty(body_empty_view) => {
                body_empty_view.set_area(area, viewport_area);
            }
        }
    }

    fn draw(&self, params: Self::Params, frame: &mut ratatui::Frame) {
        match self {
            BodyContentView::Text { editor, area } => {
                frame.render_widget(editor, *area);
            }
            BodyContentView::Binary(binary_viewer) => {
                binary_viewer.draw(params, frame);
            }
            BodyContentView::Empty(view) => {
                view.draw((), frame);
            }
        }
    }

    fn draw_overlay(&self, params: Self::Params, frame: &mut ratatui::Frame) {
        match self {
            BodyContentView::Text { .. } => {}
            BodyContentView::Binary(binary_viewer) => {
                binary_viewer.draw_overlay(params, frame);
            }
            BodyContentView::Empty(_view) => {}
        }
    }
}

pub struct BodyEmptyView {
    area: Rect,
}

impl<'params> UiComposedElement<'params> for BodyEmptyView {
    type Params = ();

    fn set_area(&mut self, area: Rect, _viewport_area: Rect) {
        self.area = area;
    }

    fn draw(&self, _params: Self::Params, frame: &mut ratatui::Frame) {
        frame.render_widget(Span::from("No body content").italic(), self.area);
    }
}
