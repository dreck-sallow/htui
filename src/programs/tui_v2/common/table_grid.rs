use ratatui::{layout::Rect, style::Style, text::Span, Frame};

use super::list;

pub struct UiTableGrid<'text, 'placeholder, const N: usize> {
    titles: [Span<'static>; N],
    rows: Vec<[Span<'text>; N]>,
    index_style: Style,
    index_cell: Option<(usize, usize)>,
    widths: [f32; N],
    placeholder_empty: Span<'placeholder>,
}

impl<'text, 'placeholder, const N: usize> UiTableGrid<'text, 'placeholder, N> {
    pub fn new(titles: [Span<'static>; N], widths: [f32; N]) -> Self {
        Self {
            titles,
            rows: Vec::new(),
            index_style: Style::default(),
            index_cell: None,
            placeholder_empty: Span::from("No items"),
            widths,
        }
    }

    pub fn with_rows(mut self, rows: Vec<[Span<'text>; N]>) -> Self {
        self.rows = rows;
        self
    }

    pub fn with_index(mut self, idx: Option<(usize, usize)>, style: Style) -> Self {
        self.index_cell = idx;
        self.index_style = style;
        self
    }

    pub fn with_placeholder(mut self, placeholder: Span<'placeholder>) -> Self {
        self.placeholder_empty = placeholder;
        self
    }
}

impl<const N: usize> UiTableGrid<'_, '_, N> {
    pub fn draw(&self, area: Rect, frame: &mut Frame) {
        if area.is_empty() {
            return;
        }

        let page_opt = list::page_list(
            &self.rows,
            self.index_cell.map(|(i, _)| i).unwrap_or(0),
            area.height as usize,
        );
        let rows = page_opt
            .map(|(start, end)| &self.rows[start..(end + 1)])
            .unwrap_or(&[]);

        let row_idx = self
            .index_cell
            .map(|(i, _)| list::in_page_cursor(i, area.height as usize))
            .unwrap_or(usize::MAX);
        let col_idx = self.index_cell.map(|(_, i)| i).unwrap_or(self.titles.len());

        for (col, col_area) in self.splits(area).iter().enumerate() {
            let mut line_area = Rect {
                height: 1,
                ..*col_area
            };
            let title = &self.titles[col];

            frame.render_widget(title, line_area);

            line_area.y += 1;

            for (row, list) in rows.iter().enumerate() {
                let itm = &list[col];
                frame.render_widget(itm, line_area);

                if row_idx == row && col_idx == col {
                    frame.buffer_mut().set_style(line_area, self.index_style);
                }

                line_area.y += 1;
            }
        }
    }

    fn splits(&self, area: Rect) -> [Rect; N] {
        let column_widths = { self.widths.map(|n| (n * (area.width as f32)) as u16) };

        let mut acc_width = area.x;
        column_widths.map(|w| {
            let left = acc_width;
            acc_width += w + 1;
            Rect {
                width: w,
                x: left,
                ..area
            }
        })
    }
}
