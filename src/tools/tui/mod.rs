use std::io::Result;

use crate::directory::Directory;
use crossterm::event::KeyCode;

mod app;
mod core;
mod elements;
mod state;
mod tui_manager;

use app::{App, AppState};

pub async fn run() -> Result<()> {
    let directory = Directory::new("com", "dreck-tui", "htui");

    let _config_file_path = directory.config_file_path().unwrap();

    let mut tui = tui_manager::TuiManager::new(4.0, 30.0)?;

    // let mut app_state = AppState::new(CollectionsState::default());
    let tui_app = App::new(AppState::default());

    tui.enter()?;

    loop {
        if let Some(ev) = tui.next().await {
            match ev {
                tui_manager::TuiEvent::Tick => {}
                tui_manager::TuiEvent::Render => {
                    tui.terminal.draw(|frame| {
                        tui_app.render(frame).unwrap();
                    })?;
                }
                tui_manager::TuiEvent::Key(k) => match k.code {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        break;
                    }
                    _ => {
                        // tui_app.update(k)?;
                    }
                },
                _ => {}
            }
        }
    }

    tui.exit()?;

    Ok(())
}
