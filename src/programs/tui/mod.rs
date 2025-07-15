use std::io;

use app::App;
use events::Events;
use ratatui::layout::Rect;
use sources::TerminalSource;
// use views::dashboard::DashboardView;

use crate::{
    paths::Paths,
    store::{models::ProjectModel, LocalStore, Store, StoreError, StoreResult},
};

mod app;
mod common;
mod element_view;
mod elements;
mod events;
mod pane;
mod sources;
// mod views;

pub async fn run_tui(project_name: Option<String>) -> io::Result<()> {
    let project = load_project(project_name).await.unwrap();

    let mut terminal = ratatui::init();

    let mut events = Events::new();
    events.add_source(TerminalSource::default());

    events.listen();

    let mut app = App::new_from_project(project, events.sender());
    app.viewport_area(Rect {
        x: 0,
        y: 0,
        width: terminal.size().unwrap().width,
        height: terminal.size().unwrap().height,
    });

    terminal.draw(|frame| {
        app.handle_draw(frame);
    })?;

    loop {
        if let Some(ev) = events.next_event().await {
            match ev {
                events::Event::Draw => {
                    terminal.draw(|frame| {
                        app.handle_draw(frame);
                    })?;
                }
                events::Event::Input(key_event) => {
                    app.handle_key(key_event);
                    terminal.draw(|frame| {
                        app.handle_draw(frame);
                    })?;
                }
                events::Event::KeyBinding(_key_event, _key_event1) => todo!(),
                events::Event::Quit => break,
            }
        }
    }

    ratatui::restore();

    Ok(())
}

async fn load_project(project_name: Option<String>) -> StoreResult<ProjectModel> {
    let local_store = LocalStore::new(Paths::new("store"));

    if let Some(name) = project_name {
        let list = local_store.project_list().await?;
        let found_itm = list.iter().find(|p| p.name == name);

        if let Some(itm) = found_itm {
            return match local_store.get_project(itm.id.clone()).await {
                Ok(p) => Ok(p),
                Err(StoreError::NotFound) => Ok(ProjectModel::new(name)),
                Err(err) => Err(err),
            };
        }

        return Ok(ProjectModel::new(name));
    }

    Ok(ProjectModel::default())
}
