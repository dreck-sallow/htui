use std::io;

use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    Terminal,
};

struct TuiManager {
    terminal: Terminal<CrosstermBackend<io::Stderr>>,
}

impl TuiManager {
    pub fn enter(&self) -> io::Result<()> {
        enable_raw_mode()?;
        execute!(io::stderr(), EnterAlternateScreen)
    }

    pub fn exit(&self) -> io::Result<()> {
        execute!(io::stderr(), LeaveAlternateScreen)?;
        disable_raw_mode()
    }
}
