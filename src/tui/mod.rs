use app_tui::AppTui;
use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    Terminal,
};
use std::io;

mod app_tui;

mod event;

use event::EventHandler;

pub type AppTuiResult = io::Result<()>;

pub async fn run_app() -> io::Result<()> {
    let mut app_tui = AppTui::new();
    let mut tui_manager = TuiManager::new()?;

    let mut event_handler = EventHandler::new(255);

    tui_manager.enter()?;

    loop {
        if let event::Event::Key(key) = event_handler.next().await.unwrap() {
            if let ratatui::crossterm::event::KeyCode::Char('q') = key.code {
                break;
            }
        }

        tui_manager.terminal.draw(|f| {
            let _ = app_tui.run(f);
        })?;
    }

    tui_manager.exit()?;
    event_handler.stop();

    Ok(())
}

struct TuiManager {
    terminal: Terminal<CrosstermBackend<io::Stderr>>,
}

impl TuiManager {
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            terminal: Terminal::new(CrosstermBackend::new(io::stderr()))?,
        })
    }
    pub fn enter(&self) -> io::Result<()> {
        enable_raw_mode()?;
        execute!(io::stderr(), EnterAlternateScreen)
    }

    pub fn exit(&self) -> io::Result<()> {
        execute!(io::stderr(), LeaveAlternateScreen)?;
        disable_raw_mode()
    }
}
