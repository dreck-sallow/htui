use std::io;

use app::TuiApp;
use events::create_events;

use crate::store::{self, models::ProjectModel, Store, StoreError};

mod app;
mod common;
mod events;
mod pane;

pub async fn run(project_name: Option<String>) -> io::Result<()> {
    let project = load_project(project_name).await.unwrap();
    let (mut events, draw_signal) = create_events();
    let mut terminal = ratatui::init();

    let mut app = TuiApp::default();
    app.add_project(project);

    events.start();
    while let Some(ev) = events.next_event().await {
        match ev {
            events::EventMsg::Draw => {
                terminal.draw(|frame| {
                    app.handle_draw(frame);
                })?;
            }
            events::EventMsg::Quit => {
                app.handle_quit();
                break;
            }
            events::EventMsg::Key(key_event) => {
                let should_continue = app.handle_key(key_event, draw_signal.clone());
                terminal.draw(|frame| {
                    app.handle_draw(frame);
                })?;

                if !should_continue {
                    break;
                }
            }
        }
    }

    events.finish();

    ratatui::restore();

    Ok(())
}

async fn load_project(project_name: Option<String>) -> store::Result<ProjectModel> {
    let store = Store::new();

    if let Some(name) = project_name {
        let list = store.list_projects().await?;
        let found_itm = list.iter().find(|p| p.name == name);

        if let Some(itm) = found_itm {
            return match store.find_one_project(itm.id.clone()).await {
                Ok(p) => Ok(p),
                Err(StoreError::NotFound) => Ok(ProjectModel::new(name)),
                Err(err) => Err(err),
            };
        }

        return Ok(ProjectModel::new(name));
    }

    Ok(ProjectModel::default())
}
