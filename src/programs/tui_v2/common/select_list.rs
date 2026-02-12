use crossterm::event::KeyEvent;
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::Span,
    Frame,
};

use super::list;

pub struct SelectList<I: 'static> {
    items: &'static [I],
    idx: Option<usize>,
    selected_style: Style,
}

impl<I> SelectList<I> {
    pub fn new(items: &'static [I]) -> Self {
        SelectList {
            items,
            selected_style: Style::default().on_dark_gray().white(),
            idx: None,
        }
    }

    // pub fn with_selected_style(mut self, style: Style) -> Self {
    //     self.selected_style = style;
    //     self
    // }

    pub fn reset(&mut self) {
        self.idx = None;
    }

    pub fn selected(&self) -> Option<&I> {
        self.idx.and_then(|i| self.items.get(i))
    }
}

impl<I: PartialEq> SelectList<I> {
    pub fn select(&mut self, select_itm: &I) {
        for (i, itm) in self.items.iter().enumerate() {
            if itm == select_itm {
                self.idx = Some(i);
                return;
            }
        }
        // TODO: set none on not found?
    }
}

impl<I> SelectList<I> {
    pub fn next(&mut self) {
        self.idx = list::next(self.idx, self.items.len());
    }

    pub fn prev(&mut self) {
        self.idx = list::prev(self.idx);
    }

    pub fn start(&mut self) {
        if self.items.is_empty() {
            self.idx = None;
        } else {
            self.idx = Some(0);
        }
    }

    pub fn end(&mut self) {
        if self.items.is_empty() {
            self.idx = None;
        } else {
            self.idx = Some(self.items.len() - 1);
        }
    }
}

impl<I: ToString> SelectList<I> {
    pub fn draw(&self, mut area: Rect, frame: &mut Frame) {
        let idx = self
            .idx
            .map(|i| list::in_page_cursor(i, area.height as usize))
            .unwrap_or(self.items.len());
        if let Some((start, end)) =
            list::page_list(self.items, self.idx.unwrap_or(0), area.height as usize)
        {
            for (i, itm) in self.items[start..(end + 1)].iter().enumerate() {
                let line = Span::raw(itm.to_string());
                frame.render_widget(line, area);

                let style = if i == idx {
                    self.selected_style
                } else {
                    Style::default()
                };

                frame
                    .buffer_mut()
                    .set_style(Rect { height: 1, ..area }, style);

                area.y += 1;
            }
        }
    }
}

impl<I> SelectList<I> {
    pub fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            crossterm::event::KeyCode::Up | crossterm::event::KeyCode::Char('k') => {
                self.prev();
            }
            crossterm::event::KeyCode::Down | crossterm::event::KeyCode::Char('j') => {
                self.next();
            }
            crossterm::event::KeyCode::Home => {
                self.start();
            }
            crossterm::event::KeyCode::End => {
                self.end();
            }
            _ => {}
        }
    }
}
