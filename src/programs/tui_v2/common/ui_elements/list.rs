use ratatui::{layout::Rect, style::Style, text::Line, Frame};

use crate::programs::tui_v2::common::list;

pub struct UiList {
    idx: Option<usize>,
    itms: Vec<Line<'static>>,
    highlight_style: Style,
}

impl UiList {
    pub fn new() -> Self {
        Self {
            idx: None,
            itms: Vec::new(),
            highlight_style: Style::default(),
        }
    }

    pub fn with_items(mut self, itms: Vec<Line<'static>>) -> Self {
        self.itms = itms;
        self
    }

    pub fn with_idx(mut self, idx: Option<usize>) -> Self {
        self.idx = idx;
        self
    }

    pub fn with_highlight_style(mut self, style: Style) -> Self {
        self.highlight_style = style;
        self
    }
}

impl UiList {
    pub fn draw(&self, area: Rect, frame: &mut Frame) {
        let Some((start_i, end_i)) =
            list::page_list(&self.itms, self.idx.unwrap_or(0), area.height as usize)
        else {
            return;
        };

        let idx = list::in_page_cursor(self.idx.unwrap_or(usize::MAX), area.height as usize);

        let mut line_area = Rect { height: 1, ..area };
        for (i, line) in self.itms[start_i..(end_i + 1)].iter().enumerate() {
            frame.render_widget(line, line_area);

            if i == idx {
                frame
                    .buffer_mut()
                    .set_style(line_area, self.highlight_style);
            }

            line_area.y += 1;
        }
    }
}
