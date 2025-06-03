use collections::{Collections, Item};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    widgets::Block,
    Frame,
};

use state::CollectionsState;

use crate::store::models::{CollectionsModel, ProjectModel, RequestModel};

use super::{
    action::Action, focus::OverlayFocus, pane_state::PaneState, upsert_item::UpsertMethod,
};

mod collections;
mod state;

pub struct CollectionsView {
    state: CollectionsState<String>,
}

impl CollectionsView {
    pub fn new() -> Self {
        Self {
            state: CollectionsState::new(),
        }
    }

    pub fn insert_collection(&mut self, collection: &CollectionsModel) {
        let children = collection
            .requests()
            .iter()
            .map(|req| req.id().to_string())
            .collect();

        self.state
            .add_collection((collection.id().to_string(), children));
    }

    // pub fn insert_request(&mut self, collection: &CollectionsModel, request: &RequestModel) {
    //     self.state
    //         .add_request(collection.id().to_string(), request.id().to_string());
    // }

    pub fn render(&self, project: &ProjectModel, frame: &mut Frame, area: Rect, is_focus: bool) {
        let items: Vec<Item<'_>> = project
            .collections()
            .iter()
            .map(|coll| {
                let mut itm = Item::new(coll.name());

                for req in coll.requests() {
                    itm.add_child(Item::new(req.name()));
                }

                itm
            })
            .collect();

        let collections = Collections::default()
            .set_items(items)
            .set_block(
                Block::bordered().title(" Collections ").border_style(
                    is_focus
                        .then_some(Style::default().blue())
                        .unwrap_or_default(),
                ),
            )
            .set_openeds(self.state.openeds())
            .set_idx(self.state.idx())
            .set_highlight_style(Style::default().green());

        frame.render_widget(collections, area);
    }

    pub fn handle_key(
        &mut self,
        key: KeyEvent,
        state: &mut PaneState,
        register_actions: &mut Vec<Action>,
    ) {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Tab => {
                    state.focus_element(super::focus::ElementFocus::RequestBuilder);
                }
                KeyCode::BackTab => {
                    state.focus_element(super::focus::ElementFocus::ResponseViewer);
                }
                KeyCode::Left | KeyCode::Char('h') => self.state.close_collection(true),
                KeyCode::Right | KeyCode::Char('l') => self.state.open_collection(true),
                KeyCode::Down | KeyCode::Char('j') => self.state.next(),
                KeyCode::Up | KeyCode::Char('k') => self.state.prev(),
                KeyCode::Char('c') => {
                    register_actions.push(Action::UpsertItem(
                        UpsertMethod::CreateCollection,
                        "".to_string(),
                    ));
                    state.focus_overlay(OverlayFocus::UpsertItem);
                }
                KeyCode::Char('r') => {
                    let idx = self.state.idx();
                    if !idx.is_none() {
                        register_actions.push(Action::UpsertItem(
                            UpsertMethod::CreateRequest,
                            "".to_string(),
                        ));
                        state.focus_overlay(OverlayFocus::UpsertItem);
                    }
                }
                KeyCode::Char('e') => {
                    let idx = self.state.idx();
                    match idx {
                        state::Idx::None => {}
                        state::Idx::Parent(i) => {
                            let name = state
                                .project_ref()
                                .collection_by_idx(i)
                                .unwrap()
                                .name()
                                .to_string();
                            register_actions
                                .push(Action::UpsertItem(UpsertMethod::EditCollection, name));
                            state.focus_overlay(OverlayFocus::UpsertItem);
                        }
                        state::Idx::Child(i, sub_i) => {
                            let name = state
                                .project_ref()
                                .request_by_idx((i, sub_i))
                                .unwrap()
                                .name()
                                .to_string();
                            register_actions
                                .push(Action::UpsertItem(UpsertMethod::EditRequest, name));
                            state.focus_overlay(OverlayFocus::UpsertItem);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    pub fn handle_action(&mut self, action: Action, state: &mut PaneState) {
        match action {
            Action::SaveUpsertItem(upsert_method, name) => match upsert_method {
                UpsertMethod::CreateRequest => {
                    let i = match self.state.idx() {
                        state::Idx::None => {
                            unreachable!()
                        }
                        state::Idx::Parent(i) => i,
                        state::Idx::Child(i, _) => i,
                    };
                    let request = RequestModel::new(name, "GET".into());
                    self.state.add_request_on_current(request.id().to_string());
                    state.project_mut().add_request_by_i(i, request);
                }
                UpsertMethod::CreateCollection => {
                    let collection = CollectionsModel::new(name);
                    self.state
                        .add_collection((collection.id().to_string(), Vec::new()));
                    state.project_mut().add_collection(collection);
                }
                _ => {}
            },
            _ => {}
        }
    }
}
