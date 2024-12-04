use std::io::Result;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders},
    Frame,
};

pub struct TuiApp {
    pub name: String,
}

impl TuiApp {
    pub fn new() -> Self {
        Self {
            name: String::new(),
        }
    }

    pub fn render(&self, frame: &mut Frame) -> Result<()> {
        let chunk = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
            .split(frame.area());

        let tree_block = Block::default().title("tree folder").borders(Borders::ALL);
        frame.render_widget(tree_block, chunk[0]);

        let content_block = Block::default()
            .title("content block")
            .borders(Borders::ALL);
        frame.render_widget(content_block, chunk[1]);

        // execute!(stdout(), Print(self.name.clone()))?;
        Ok(())
    }

    pub fn update(&mut self, key: KeyEvent) -> Result<()> {
        if let KeyCode::Char(ch) = key.code {
            self.name.push(ch)
        }
        Ok(())
    }
}
