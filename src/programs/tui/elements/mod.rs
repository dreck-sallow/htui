use ratatui::{style::Style, symbols, widgets::Widget};

pub mod dropdown;
pub mod table;

pub mod utils {
    use ratatui::layout::{Constraint, Layout, Rect};

    pub fn expand(txt: &str, fill: &str, width: usize) -> String {
        let missing_len = width - txt.chars().count();

        let left_half = missing_len / 2;
        let right_half = (missing_len) - left_half;

        let left = &fill.repeat(left_half);
        let right = &fill.repeat(right_half);

        format!("{left}{txt}{right}")
    }

    pub fn center_area(total_area: Rect, height: Constraint, width: Constraint) -> Rect {
        let [area] = Layout::vertical([height])
            .flex(ratatui::layout::Flex::Center)
            .areas(total_area);

        let [area] = Layout::horizontal([width])
            .flex(ratatui::layout::Flex::Center)
            .areas(area);

        area
    }
}

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
