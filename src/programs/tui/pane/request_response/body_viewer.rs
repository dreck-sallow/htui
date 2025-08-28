use std::rc::Rc;

use crate::programs::tui::{
    common::component::Drawable, config::Config, pane::text_editor::TextEditor,
};
use ratatui::{layout::Rect, style::Stylize, text::Span};

use super::binary_viewer::BinaryViewer;

pub enum BodyContentView {
    Text(TextEditor),
    Binary(BinaryViewer),
    Empty,
}

impl Drawable for BodyContentView {
    type Params = (Rect, Rc<Config>);

    fn draw<'a: 'painter, 'painter>(
        &'a self,
        painter: &mut crate::programs::tui::common::component::Painter<'painter>,
        (area, config): Self::Params,
    ) {
        match self {
            BodyContentView::Text(text_editor) => {
                painter.render(move |frame| {
                    frame.render_widget(text_editor, area);
                });
            }
            BodyContentView::Binary(binary_viewer) => {
                binary_viewer.draw(painter, (area, config));
            }
            BodyContentView::Empty => {
                if area.is_empty() {
                    return;
                }

                painter.render(move |frame| {
                    frame.render_widget(Span::from("No body content").italic(), area);
                });
            }
        }
    }
}

// impl Widget for &BodyContentView {
//     fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
//     where
//         Self: Sized,
//     {
//         match self {
//             BodyContentView::Text(text_editor) => text_editor.render(area, buf),
//             BodyContentView::Binary(binary_viewer) => binary_viewer.render(area, buf),
//             BodyContentView::Empty => {
//                 if area.is_empty() {
//                     return;
//                 }

//                 buf.set_span(
//                     area.left(),
//                     area.top(),
//                     &Span::from("No body content").italic(),
//                     area.width,
//                 );
//             }
//         }
//     }
// }
