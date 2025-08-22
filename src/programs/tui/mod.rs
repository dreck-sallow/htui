use std::{cell::RefCell, io, rc::Rc};

use app::App;
use config::load_config;
use ratatui::layout::Rect;
use sources::TerminalSourceV2;

use crate::app_project::{
    self,
    models::ProjectModel,
    paths::ProjectPaths,
    store::{LocalStore, Store, StoreError},
};

mod app;
mod app_components;
mod common;
mod config;
mod elements;
mod event_handler;
mod pane;
mod sources;

#[derive(Debug)]
pub enum TuiError {
    Io(io::Error),
    Config(toml::de::Error),
}

impl From<io::Error> for TuiError {
    fn from(value: io::Error) -> Self {
        TuiError::Io(value)
    }
}

impl From<toml::de::Error> for TuiError {
    fn from(value: toml::de::Error) -> Self {
        TuiError::Config(value)
    }
}

pub type TuiResult<T> = Result<T, TuiError>;

pub async fn run_tui(project_name: Option<String>) -> TuiResult<()> {
    let project = load_project(project_name).unwrap();
    let config = load_config(&ProjectPaths::new())?;

    let terminal = Rc::new(RefCell::new(ratatui::init()));

    let events = Rc::new(RefCell::new(event_handler::Events::from_sources(vec![
        Box::new(TerminalSourceV2::new()),
    ])));

    events.borrow_mut().run();

    let mut app = App::new_from_project(project, Rc::new(config), events.borrow().sender());
    app.viewport_area(Rect {
        x: 0,
        y: 0,
        width: terminal.borrow().size().unwrap().width,
        height: terminal.borrow().size().unwrap().height,
    });

    terminal.borrow_mut().draw(|frame| {
        app.handle_draw(frame);
    })?;

    loop {
        let ev_result = events.borrow_mut().next_message().await;

        match ev_result {
            Some(ev) => match ev {
                event_handler::AppMessage::Draw => {
                    terminal.borrow_mut().draw(|frame| {
                        app.handle_draw(frame);
                    })?;
                }
                event_handler::AppMessage::Input(key_event) => {
                    // let value = events.try_borrow_mut();
                    app.handle_key(key_event, Rc::clone(&events), Rc::clone(&terminal));
                    terminal.borrow_mut().draw(|frame| {
                        app.handle_draw(frame);
                    })?;
                }
                event_handler::AppMessage::Quit => break,
            },
            None => {}
        }
    }

    ratatui::restore();

    Ok(())
}

fn load_project(project_name: Option<String>) -> app_project::store::Result<ProjectModel> {
    let local_store = LocalStore::new();

    if let Some(name) = project_name {
        let list = local_store.project_list()?;
        let found_itm = list.iter().find(|p| p.name == name);

        if let Some(itm) = found_itm {
            return match local_store.get_project(itm.id.clone()) {
                Ok(p) => Ok(p),
                Err(StoreError::NotFound) => Ok(ProjectModel::new(name)),
                Err(err) => Err(err),
            };
        }

        return Ok(ProjectModel::new(name));
    }

    Ok(ProjectModel::default())
}
