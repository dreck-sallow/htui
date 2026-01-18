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

pub fn ui_placeholder(line: &str) -> PlaceholderLine {
    PlaceholderLine::new(line).with_style(Style::default().dark_gray().italic())
}
