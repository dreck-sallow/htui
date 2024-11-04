use std::io;
use std::panic;
use std::time::Duration;

use event_handler::AppEventHandler;
use ratatui::crossterm::style::Print;
use ratatui::crossterm::terminal::disable_raw_mode;
use ratatui::crossterm::terminal::EnterAlternateScreen;
use ratatui::crossterm::terminal::LeaveAlternateScreen;
use ratatui::crossterm::{execute, terminal::enable_raw_mode};
use tokio::time::sleep;

mod event_handler;

pub async fn run_app() -> io::Result<()> {
    set_panic_hook();

    // Init setup for terminal
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    execute!(io::stdout(), Print("printed!".to_string()))?;

    let mut event_handler = AppEventHandler::new(4);

    sleep(Duration::from_secs(3)).await;
    event_handler.stop().await;

    // Restore the terminal state, for leaving the app
    execute!(io::stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()
}

fn set_panic_hook() {
    let hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_hook| {
        execute!(io::stdout(), LeaveAlternateScreen).unwrap();
        disable_raw_mode().unwrap();
        hook(panic_hook)
    }));
}
