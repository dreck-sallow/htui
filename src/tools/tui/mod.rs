use std::io::Result;

use crate::directory;
use crossterm::event::KeyCode;

mod app;
mod core;
mod element;
mod elements;
mod state;
mod store;
mod tui_manager;

use app::{App, AppState};
use elements::collections::{CollectionItem, CollectionState, RequestItem};
use store::{project::Project, Store};

pub async fn run(project_name: Option<String>) -> Result<()> {
    let directory = directory::DirectoryV2::from("com", "dreck-htui", "htui").unwrap();

    let store = Store::new(
        directory.data_file_path().unwrap(),
        directory.data_path().clone(),
    );

    let project = match project_name {
        Some(name) if !name.is_empty() => store.get_or_create_project(&name).unwrap(),
        _ => Project::default(),
    };

    let mut tui = tui_manager::TuiManager::new(4.0, 30.0)?;

    let mut collection_state = CollectionState::default();

    for collection in project.collections {
        let requests: Vec<RequestItem> = collection
            .requests
            .iter()
            .map(|req| RequestItem::new(req.name.clone(), req.method.clone(), req.url.clone()))
            .collect();

        collection_state.add_item(CollectionItem::new(collection.name.clone()), requests);
    }

    let app_state = AppState::default().with_collections(collection_state);

    let mut tui_app = App::new(app_state);

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
                        tui_app.handle(&k);
                    }
                },
                _ => {}
            }
        }
    }

    tui.exit()?;

    Ok(())
}
