use std::io::{stdout, Result};

use crossterm::{event::KeyCode, execute, style::Print};

mod tui_manager;

pub async fn run() -> Result<()> {
    let mut tui = tui_manager::TuiManager::new(4.0, 30.0)?;

    tui.enter()?;

    loop {
        tui.terminal.draw(|frame| {})?;

        if let Some(ev) = tui.next().await {
            match ev {
                tui_manager::TuiEvent::Tick => {
                    execute!(stdout(), Print("\ntick\n"))?;
                }
                tui_manager::TuiEvent::Render => {
                    execute!(stdout(), Print("render"))?;
                }
                tui_manager::TuiEvent::Key(k) => match k.code {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        execute!(stdout(), Print("\n EXIT \n"))?;
                        break;
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    tui.exit()?;

    Ok(())
}
