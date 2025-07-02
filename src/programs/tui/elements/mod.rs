use ratatui::{style::Style, symbols, widgets::Widget};

pub struct Separator {
    symbol: &'static str,
    style: Style, // SUGGEST: use only fg & bg?
}

impl Default for Separator {
    fn default() -> Self {
        Self {
            symbol: symbols::block::ONE_EIGHTH,
            style: Style::default(),
        }
    }
}

impl Separator {
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn symbol(mut self, symbol: &'static str) -> Self {
        self.symbol = symbol;
        self
    }
}

impl Widget for Separator {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        if area.is_empty() {
            return;
        }

        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                buf.set_stringn(x, y, self.symbol, self.symbol.len(), self.style);
            }
        }
    }
}
