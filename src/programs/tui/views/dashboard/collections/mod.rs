use collections::{Collections, Item};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    widgets::Block,
    Frame,
};

use crate::programs::tui::element_view::ElementView;

use super::{
    global_pane_state::{CollectionChange, GlobalPaneState},
    upsert_item::UpsertMethod,
};

mod collections;
mod state;

pub use state::{CollectionsState, Idx};

pub struct CollectionsView {
    render_area: Rect,
}

impl CollectionsView {
    pub fn new() -> Self {
        Self {
            render_area: Rect::default(),
        }
    }
}

impl ElementView for CollectionsView {
    type State = GlobalPaneState;

    fn draw(&self, frame: &mut Frame, state: &Self::State) {
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

        frame.render_widget(collections, self.render_area);
    }

    fn set_area(&mut self, area: Rect) {
        self.render_area = area;
    }

    fn on_key(&mut self, key: KeyEvent, state: &mut Self::State) {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Tab => {
                    state.set_focus(super::focus::ElementFocus::MethodUrlBar);
                }
                KeyCode::Enter => {
                    state.collection_change(CollectionChange::Select);
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
                KeyCode::Char('d') | KeyCode::Delete => {
                    let (_, idx) = state.collections_raw_data();

                    match idx {
                        state::Idx::None => {}
                        state::Idx::Parent(i) => {
                            state.delete_collection_item((i, None));
                        }
                        state::Idx::Child(i, sub_i) => {
                            state.delete_collection_item((i, Some(sub_i)));
                        }
                    }
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

    fn on_change_state(&mut self, _state: &Self::State) {}
}
