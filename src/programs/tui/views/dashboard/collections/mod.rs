use collections::{Collections, Item};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    widgets::Block,
    Frame,
};

use super::{
    global_pane_state::{CollectionChange, GlobalPaneState},
    upsert_item::UpsertMethod,
};

mod collections;
mod state;

pub use state::{CollectionsState, Idx};

pub struct CollectionsView;

impl CollectionsView {
    pub fn new() -> Self {
        Self {}
    }

    // pub fn insert_collection(&mut self, collection: &CollectionsModel) {
    //     let children = collection
    //         .requests()
    //         .iter()
    //         .map(|req| req.id().to_string())
    //         .collect();

    //     self.state
    //         .add_collection((collection.id().to_string(), children));
    // }

    // pub fn insert_request(&mut self, collection: &CollectionsModel, request: &RequestModel) {
    //     self.state
    //         .add_request(collection.id().to_string(), request.id().to_string());
    // }

    pub fn render(&self, frame: &mut Frame, area: Rect, state: &GlobalPaneState) {
        let items: Vec<Item<'_>> = state
            .project_collections()
            .iter()
            .map(|coll| {
                let mut itm = Item::new(coll.name());

                for req in coll.requests() {
                    itm.add_child(Item::new(req.name()));
                }

                itm
            })
            .collect();

        let is_focus = state.is_focus(super::focus::ElementFocus::Collections);
        let (openeds, idx) = state.collections_raw_data();

        let collections = Collections::default()
            .set_items(items)
            .set_block(
                Block::bordered().title(" Collections ").border_style(
                    is_focus
                        .then_some(Style::default().blue())
                        .unwrap_or_default(),
                ),
            )
            .set_openeds(openeds)
            .set_idx(idx)
            .set_highlight_style(Style::default().green());

        frame.render_widget(collections, area);
    }

    pub fn handle_key(&mut self, key: KeyEvent, state: &mut GlobalPaneState) {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Tab => {
                    state.set_focus(super::focus::ElementFocus::RequestBuilder);
                }
                KeyCode::BackTab => {
                    state.set_focus(super::focus::ElementFocus::ResponseViewer);
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    state.collection_change(CollectionChange::CloseCollection)
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    state.collection_change(CollectionChange::OpenCollection)
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    state.collection_change(CollectionChange::NextItem)
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    state.collection_change(CollectionChange::PrevItem)
                }
                KeyCode::Char('c') => {
                    state.set_upsert_form(UpsertMethod::CreateCollection("".into()));
                    state.set_overlay(super::focus::OverlayFocus::UpsertItem);
                }
                KeyCode::Char('r') => {
                    let (_, idx) = state.collections_raw_data();
                    if !idx.is_none() {
                        state.set_upsert_form(UpsertMethod::CreateRequest("".into()));
                        state.set_overlay(super::focus::OverlayFocus::UpsertItem);
                    }
                }
                KeyCode::Char('e') => {
                    let (_, idx) = state.collections_raw_data();
                    match idx {
                        state::Idx::None => {}
                        state::Idx::Parent(i) => {
                            let name = state
                                .project_ref()
                                .collection_by_idx(i)
                                .unwrap()
                                .name()
                                .to_string();
                            state.set_upsert_form(UpsertMethod::EditCollection(name));
                            state.set_overlay(super::focus::OverlayFocus::UpsertItem);
                        }
                        state::Idx::Child(i, sub_i) => {
                            let name = state
                                .project_ref()
                                .request_by_idx((i, sub_i))
                                .unwrap()
                                .name()
                                .to_string();
                            state.set_upsert_form(UpsertMethod::EditRequest(name));
                            state.set_overlay(super::focus::OverlayFocus::UpsertItem);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // pub fn handle_action(&mut self, action: Action, state: &mut PaneState) {
    //     match action {
    //         Action::SaveUpsertItem(upsert_method, name) => match upsert_method {
    //             UpsertMethod::CreateRequest => {
    //                 let i = match self.state.idx() {
    //                     state::Idx::None => {
    //                         unreachable!()
    //                     }
    //                     state::Idx::Parent(i) => i,
    //                     state::Idx::Child(i, _) => i,
    //                 };
    //                 let request = RequestModel::new(name);
    //                 self.state.add_request_on_current(request.id().to_string());
    //                 state.project_mut().add_request_by_i(i, request);
    //             }
    //             UpsertMethod::CreateCollection => {
    //                 let collection = CollectionsModel::new(name);
    //                 self.state
    //                     .add_collection((collection.id().to_string(), Vec::new()));
    //                 state.project_mut().add_collection(collection);
    //             }
    //             _ => {}
    //         },
    //         _ => {}
    //     }
    // }
}
