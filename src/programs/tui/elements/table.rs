use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::Span,
    widgets::Widget,
};

pub struct TableGrid<'text, 'placeholder, const N: usize> {
    titles: [Span<'static>; N],
    rows: Vec<[Span<'text>; N]>,
    index_style: Style,
    index_cell: Option<(usize, usize)>,
    widths: [f32; N],
    placeholder_empty: Span<'placeholder>,
}

impl<'text, 'placeholder, const N: usize> TableGrid<'text, 'placeholder, N> {
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

impl<const N: usize> Widget for TableGrid<'_, '_, N> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        if area.is_empty() {
            return;
        }

        let column_widths = { self.widths.map(|n| (n * (area.width as f32)) as u16) };

        // Draw headers
        {
            let header_area = Rect { height: 1, ..area };
            let mut left = header_area.left();
            for (i, title) in self.titles.iter().enumerate() {
                let width = column_widths[i];
                buf.set_span(left, header_area.top(), title, width);
                left += width + 1;
            }
        }

        if self.rows.is_empty() {
            buf.set_span(
                area.left(),
                area.top() + 1,
                &self.placeholder_empty,
                area.width,
            );
        } else {
            // draw content rows
            let (index_start, index_end) = {
                let height = area.height.saturating_sub(1);
                // TODO: loop on items when support multine on table cell text
                let (start_page_idx, end_page_idx) = match self.index_cell {
                    Some((i, _)) => {
                        let in_page_idx = i as u16 / height;
                        let start = in_page_idx * height;
                        (start, start + height)
                    }
                    None => (0, height),
                };

                let max_end = if end_page_idx as usize > self.rows.len() {
                    self.rows.len() as u16
                } else {
                    end_page_idx
                };

                (start_page_idx as usize, max_end as usize)
            };

            let (mut top, mut left) = (area.top() + 1, area.left());

            for (row_i, row_values) in self.rows[index_start..index_end].iter().enumerate() {
                for (col_i, text) in row_values.iter().enumerate() {
                    let width = column_widths[col_i];

                    let show_text = if text.width() == 0 {
                        &Span::raw("-").italic()
                    } else {
                        text
                    };

                    buf.set_span(left, top, show_text, width);

                    // check if the current cell is cursor focus
                    let is_focus_cell = match self.index_cell {
                        Some(idx) => {
                            // We need get the position on the visible slice where is the
                            // row focused
                            let focus_row_in_slice_page = idx.0 - index_start;

                            row_i == focus_row_in_slice_page && col_i == idx.1
                        }
                        None => false,
                    };

                    if is_focus_cell {
                        buf.set_style(
                            Rect {
                                x: left,
                                y: top,
                                width,
                                height: 1, // TODO: change for multiline
                            },
                            self.index_style,
                        );
                    }

                    // update the left, for the next cell
                    let spacing = 1;
                    left += width + spacing;
                }

                top += 1;
                left = area.left();
            }
        }
    }
}

/// Index controller created to work with TableGrid
pub struct TableGridIndex {
    index: Option<(usize, usize)>,
}

impl TableGridIndex {
    pub fn new(len: usize) -> Self {
        Self {
            index: if len == 0 { None } else { Some((0, 0)) },
        }
    }

    pub fn index(&self) -> Option<(usize, usize)> {
        self.index
    }

    pub fn next_row(&mut self, rows: usize) {
        let is_empty = rows == 0;

        match self.index {
            Some((row_i, col_i)) => {
                if row_i < rows.saturating_sub(1) {
                    self.index = Some((row_i + 1, col_i))
                }
            }
            None => {
                if !is_empty {
                    self.index = Some((0, 0))
                }
            }
        }
    }

    pub fn previous_row(&mut self) {
        if let Some((row_i, col_i)) = self.index {
            if row_i > 0 {
                self.index = Some((row_i - 1, col_i))
            }
        }
    }

    pub fn next_col(&mut self, cols: usize) {
        if let Some((row_i, col_i)) = self.index {
            if col_i < cols.saturating_sub(1) {
                self.index = Some((row_i, col_i + 1));
            }
        }
    }

    pub fn previous_col(&mut self) {
        if let Some((row_i, col_i)) = self.index {
            if col_i > 0 {
                self.index = Some((row_i, col_i - 1));
            }
        }
    }
}
