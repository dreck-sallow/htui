use std::io;

use app::{TaskGroupKey, TuiApp};
use events::create_events;
use task::Tasks;

use crate::{
    http::set_http_client,
    store::{self, models::ProjectModel, Store, StoreError},
};

mod app;
mod app_event;
mod common;
mod ctx;
mod events;
mod pane;
mod task;

pub async fn run(project_name: Option<String>) -> io::Result<()> {
    set_http_client();

    let project = load_project(project_name).await.unwrap();

    let (mut events, draw_signal) = create_events();
    let (mut rx_tasks, tasks) = Tasks::<TaskGroupKey>::setup();

    let mut terminal = ratatui::init();

    let ctx = ctx::InitialCtx {
        draw_signal: draw_signal.clone(),
        task_sender: tasks.sender_for_group(project.id.clone()),
    };

    let mut app = TuiApp::new_from_project(project, ctx).await;

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
            Some(task) = rx_tasks.recv() => {
                app.handle_task(task);
                terminal.draw(|frame| {
                    app.handle_draw(frame);
                })?;
            }
            else => {
                break;
            }
        }
    }

    tasks.stop();

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
