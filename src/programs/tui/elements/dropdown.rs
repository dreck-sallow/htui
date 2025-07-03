use std::{cell::RefCell, rc::Rc};

use ratatui::{
    layout::{Margin, Rect},
    style::{Style, Stylize},
    widgets::{Block, Borders, Clear},
    Frame,
};

/// Type alias for shared dropdown inner data
pub type SharedDropdown<T> = Rc<RefCell<OverlayDropdownData<T>>>;

pub struct OverlayDropdownData<T> {
    selected: T,
    area: Rect,
}

impl<T> OverlayDropdownData<T> {
    pub fn new(current: T) -> Self {
        Self {
            selected: current,
            area: Rect::default(),
        }
    }

    pub fn select(&mut self, current: T) {
        self.selected = current;
    }

    pub fn set_area(&mut self, area: Rect) {
        self.area = area;
    }
}

pub struct OverlayDropdown<T> {
    data: SharedDropdown<T>,
    items: Vec<T>,
    highlight_style: Style,
}

impl<T: PartialEq + Copy + Clone> OverlayDropdown<T> {
    pub fn with_items<Items: Into<Vec<T>>>(
        data: Rc<RefCell<OverlayDropdownData<T>>>,
        items: Items,
    ) -> Self {
        Self {
            data,
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
        let selected = self.data.borrow().selected;

        for (i, itm) in self.items.iter().enumerate() {
            if selected == *itm {
                idx = i;
            }
        }

        idx
    }

    pub fn selected(&self) -> T {
        self.data.borrow().selected
    }

    pub fn next(&mut self) {
        let items_len = self.items.len();
        let idx = self.idx();

        if idx < items_len - 1 {
            self.data.borrow_mut().selected = self.items[idx + 1];
        }
    }

    pub fn prev(&mut self) {
        let idx = self.idx();
        if idx > 0 {
            self.data.borrow_mut().selected = self.items[idx - 1];
        }
    }
}

impl<T: PartialEq + AsRef<str> + Copy> OverlayDropdown<T> {
    pub fn draw(&self, frame: &mut Frame) {
        let render_area = self.data.borrow().area;
        frame.render_widget(Clear, render_area);
        frame.render_widget(
            Block::new()
                .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
                .border_style(Style::default().blue()),
            render_area,
        );

        let mut area = render_area.inner(Margin::new(1, 0));

        let buf = frame.buffer_mut();

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
