use std::io::Result;

use app::TuiApp;
use crossterm::event::KeyCode;

mod app;
mod tui_manager;

pub async fn run() -> Result<()> {
    let mut tui = tui_manager::TuiManager::new(4.0, 30.0)?;

    let mut tui_app = TuiApp::new();

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
                        tui_app.update(k)?;
                    }
                },
                _ => {}
            }
        }
    }

    tui.exit()?;

    Ok(())
}
