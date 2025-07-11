use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Margin, Rect},
    style::{Style, Stylize},
    widgets::{Block, Borders, Clear, Widget},
};

pub struct OverlayDropdown_v2<T> {
    selected: T,
    items: Vec<T>,
    highlight_style: Style,
}

impl<T> OverlayDropdown_v2<T> {
    pub fn select(&mut self, current: T) {
        self.selected = current;
    }

    pub fn selected(&self) -> &T {
        &self.selected
    }
}

impl<T: PartialEq + Copy + Clone> OverlayDropdown_v2<T> {
    pub fn with_items<Items: Into<Vec<T>>>(selected: T, items: Items) -> Self {
        Self {
            selected,
            items: items.into(),
            highlight_style: Style::default(),
        }
    }

    pub fn with_highlight_style(mut self, style: Style) -> Self {
        self.highlight_style = style;
        self
    }

    #[inline]
    pub fn idx(&self) -> usize {
        let mut idx = 0;
        for (i, itm) in self.items.iter().enumerate() {
            if self.selected == *itm {
                idx = i;
            }
        }

        idx
    }

    pub fn next(&mut self) {
        let items_len = self.items.len();
        let idx = self.idx();

        if idx < items_len - 1 {
            self.selected = self.items[idx + 1];
        }
    }

    pub fn prev(&mut self) {
        let idx = self.idx();
        if idx > 0 {
            self.selected = self.items[idx - 1];
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.next();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.prev();
            }
            _ => {}
        }
    }
}

impl<T: PartialEq + Copy + Clone + AsRef<str>> Widget for &OverlayDropdown_v2<T> {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let render_area = area;
        Clear.render(render_area, buf);
        Block::new()
            .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
            .border_style(Style::default().blue())
            .render(area, buf);

        let mut area = render_area.inner(Margin::new(1, 0));

        if area.is_empty() {
            return;
        }

        let bottom = area.bottom();
        area.height = 1;

        for (i, item) in self.items.iter().enumerate() {
            if area.y >= bottom {
                break;
            }

            let style = if self.idx() == i {
                self.highlight_style
            } else {
                Style::default()
            };

            let label = item.as_ref();

            buf.set_stringn(area.x, area.y, label, label.len(), style);
            buf.set_style(area, style);
            area.y += 1;
        }
    }
}

pub struct OverlayDropdown<T> {
    selected: T,
    items: Vec<T>,
    highlight_style: Style,
    style: Style,
}

impl<T> OverlayDropdown<T> {
    pub fn select(&mut self, current: T) {
        self.selected = current;
    }

    pub fn selected(&self) -> &T {
        &self.selected
    }
}

impl<T: PartialEq + Copy + Clone> OverlayDropdown<T> {
    pub fn with_items<Items: Into<Vec<T>>>(selected: T, items: Items) -> Self {
        Self {
            selected,
            items: items.into(),
            highlight_style: Style::default(),
            style: Style::default(),
        }
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn with_highlight_style(mut self, style: Style) -> Self {
        self.highlight_style = style;
        self
    }

    #[inline]
    pub fn idx(&self) -> usize {
        let mut idx = 0;
        for (i, itm) in self.items.iter().enumerate() {
            if self.selected == *itm {
                idx = i;
            }
        }

        idx
    }

    pub fn next(&mut self) {
        let items_len = self.items.len();
        let idx = self.idx();

        if idx < items_len - 1 {
            self.selected = self.items[idx + 1];
        }
    }

    pub fn prev(&mut self) {
        let idx = self.idx();
        if idx > 0 {
            self.selected = self.items[idx - 1];
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.next();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.prev();
            }
            _ => {}
        }
    }
}

impl<T: PartialEq + Copy + Clone + AsRef<str>> Widget for &OverlayDropdown<T> {
    fn render(self, mut area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        Clear.render(area, buf);

        if area.is_empty() {
            return;
        }

        buf.set_style(area, self.style);

        let bottom = area.bottom();
        area.height = 1;

        for (i, item) in self.items.iter().enumerate() {
            if area.y >= bottom {
                break;
            }

            let style = if self.idx() == i {
                self.highlight_style
            } else {
                Style::default()
            };

            let label = item.as_ref();

            buf.set_stringn(area.x, area.y, label, label.len(), style);
            buf.set_style(area, style);
            area.y += 1;
        }
    }
}
