use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::Span,
    widgets::Widget,
};

pub struct ParamsTable<'text> {
    header: [&'static str; 3],
    key_values: Vec<[Span<'text>; 3]>,
    title_style: Style,
    index_style: Option<Style>,
    index_cell: Option<(usize, usize)>,
}

impl<'text> ParamsTable<'text> {
    pub fn new(items: Vec<[Span<'text>; 3]>) -> Self {
        let items_len = items.len();
        Self {
            header: ["Enable", "Key", "Value"],
            key_values: items,
            title_style: Style::default(),
            index_style: None,
            index_cell: if items_len > 0 { Some((0, 0)) } else { None },
        }
    }
}

impl<'a> Widget for ParamsTable<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        if area.is_empty() {
            return;
        }

        let header_area = Rect { height: 1, ..area };
        let content_area = Rect {
            y: area.y + 1,
            ..area
        };

        let colum_widths = {
            let missing_width = area.width - 6;
            let key_width = ((missing_width as f32) * 0.4) as u16;

            [6, key_width, missing_width - key_width]
        };
        // Draw the header (titles)

        for (i, title) in self.header.iter().enumerate() {
            let width = colum_widths[i];
            buf.set_stringn(
                header_area.left() + (i as u16 * width),
                header_area.top(),
                title,
                width as usize,
                self.title_style,
            );
        }

        // If no items, render a placeholder

        if self.key_values.is_empty() {
            let msg = "No items";
            buf.set_stringn(
                content_area.left(),
                content_area.top(),
                msg,
                msg.len(),
                Style::default().gray(),
            );
        } else {
            // Draw the list table content
            // get the visible page
            let (page_start, page_end) = {
                let in_page = |start_i: usize| {
                    let height = content_area.height;
                    let mut acc_height = 0;

                    let mut end_i = start_i;

                    for _ in &self.key_values[start_i..] {
                        // Handle multilines?
                        acc_height += 1;

                        if acc_height > height {
                            break;
                        }

                        end_i += 1;
                    }

                    (start_i, end_i)
                };

                match self.index_cell {
                    Some((row_i, _)) => loop {
                        let idx = in_page(0);
                        if row_i >= idx.0 && row_i <= idx.1 {
                            break idx;
                        }
                    },
                    None => in_page(0),
                }
            };

            let mut top = content_area.top();

            for key_value in &self.key_values[page_start..(page_end + 1)] {
                for (i, text) in key_value.iter().enumerate() {
                    let width = colum_widths[i];
                    buf.set_span(i as u16 * width, top, text, width);
                }

                top += 1;
            }
        }
    }
}
