use std::{io, sync::OnceLock};

use app::TuiApp;
use app_event::create_background_tasks;
use events::create_events;

use crate::store::{self, models::ProjectModel, Store, StoreError};

mod app;
mod app_event;
mod common;
mod events;
mod pane;

static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

pub async fn run(project_name: Option<String>) -> io::Result<()> {
    let client = reqwest::ClientBuilder::new().build();
    match client {
        Ok(client) => {
            HTTP_CLIENT.get_or_init(move || client);
        }
        Err(_e) => return Err(io::Error::other("http not supported?")),
    }

    let project = load_project(project_name).await.unwrap();

    let (mut events, draw_signal) = create_events();
    let (sender_tasks, mut bg_tasks) = create_background_tasks();

    let mut terminal = ratatui::init();

    let mut app = TuiApp::default();
    app.add_project_v2(project, draw_signal.clone(), sender_tasks)
        .await;

    events.start();

    loop {
        tokio::select! {
            Some(ev) = events.next_event() => {
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
                        let should_continue = app.handle_key(key_event).await;
                        terminal.draw(|frame| {
                           app.handle_draw(frame);
                        })?;
                        if !should_continue {
                            break;
                        }
                    }
                }

            }
            Some(ev) = bg_tasks.next_event() => {
                match ev.event {
                    app_event::Event::Response { req_id, response } => {
                    let should_continue = app.handle_response(ev.pane_id, req_id, response);
                        terminal.draw(|frame| {
                           app.handle_draw(frame);
                        })?;
                    },
                }
            }
            else => {
                break;
            }
        }
    }

    bg_tasks.stop();

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
