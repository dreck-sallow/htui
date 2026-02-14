use ratatui::{
    style::{Color, Style, Stylize},
    widgets::{block::Title, Block, BorderType},
};

use super::placeholder::PlaceholderLine;

pub fn ui_block<'t, T>(title: T, is_focus: bool) -> Block<'t>
where
    T: Into<Title<'t>>,
{
    Block::bordered()
        .title(title)
        .border_type(if is_focus {
            BorderType::Thick
        } else {
            BorderType::Plain
        })
        .border_style(
            Style::default().fg(is_focus.then_some(Color::Blue).unwrap_or(Color::DarkGray)),
        )
}

pub fn ui_highlight() -> Style {
    Style::default().fg(Color::Blue)
}

pub fn ui_placeholder(line: &str) -> PlaceholderLine {
    PlaceholderLine::new(line).with_style(Style::default().dark_gray().italic())
}

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
