use std::{collections::HashSet, ops::Not};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use list::{CollectionList, Item};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, Clear},
};
use state::Idx;
use tui_textarea::Input;
use upsert_item::{UpsertItemPopup, UpsertMethod};

use crate::{
    programs::tui::element_view::Painter,
    store::models::{CollectionsModel, SendRequestId},
};

use super::store::{actions::Action, PaneStore};

mod list;
pub mod state;
mod upsert_item;

pub struct CollectionsView {
    render_area: Rect,
    menu: UpsertItemPopup,
    show_popup: bool,
    openeds: HashSet<usize>,
    idx: Idx,
}

impl CollectionsView {
    pub fn new(idx: Idx) -> Self {
        Self {
            render_area: Rect::default(),
            menu: UpsertItemPopup::new(),
            show_popup: false,
            openeds: HashSet::new(),
            idx,
        }
    }

    fn next_collection(&mut self, collections: &[CollectionsModel]) {
        let idx = match self.idx {
            Idx::None => collections.is_empty().not().then_some(0),
            Idx::Parent(i) | Idx::Child(i, _) => (i < collections.len() - 1).then_some(i + 1),
        };

        if let Some(i) = idx {
            self.idx = Idx::Parent(i);
        }
    }

    fn next_request(&mut self, collections: &[CollectionsModel]) {
        let child_idx = match self.idx {
            Idx::None => collections
                .first()
                .and_then(|coll| (!coll.requests.is_empty()).then_some((0, 0))),
            Idx::Parent(i) => {
                if self.openeds.contains(&i) {
                    let coll = &collections[i];
                    (!coll.requests.is_empty()).then_some((i, 0))
                } else {
                    None
                }
            }
            Idx::Child(i, sub_i) => {
                let coll = &collections[i];
                (sub_i < coll.requests.len() - 1).then_some((i, sub_i + 1))
            }
        };

        if let Some((i, sub_i)) = child_idx {
            self.idx = Idx::Child(i, sub_i);
        }
    }

    fn next(&mut self, collections: &[CollectionsModel]) {
        match self.idx {
            Idx::None => self.next_collection(collections),
            _ => {
                let previous_idx = self.idx.clone();
                self.next_request(collections);

                if self.idx == previous_idx {
                    self.next_collection(collections);
                }
            }
        }
    }

    fn prev_collection(&mut self) {
        let idx = match self.idx {
            Idx::None => None,
            Idx::Parent(i) => (i > 0).then(|| i - 1),
            Idx::Child(i, _) => Some(i),
        };

        if let Some(i) = idx {
            self.idx = Idx::Parent(i);
        }
    }

    fn prev_request(&mut self, collections: &[CollectionsModel]) {
        let idx = match self.idx {
            Idx::None => None,
            Idx::Parent(i) => i
                .checked_sub(1)
                .and_then(|prev_i| self.openeds.contains(&prev_i).then_some(prev_i))
                .and_then(|prev_i| Some((prev_i, &collections[prev_i])))
                .and_then(|(prev_i, coll)| {
                    (!coll.requests.is_empty()).then(|| (prev_i, coll.requests.len() - 1))
                }),
            Idx::Child(i, sub_i) => (sub_i > 0).then(|| (i, sub_i - 1)),
        };

        if let Some((i, sub_i)) = idx {
            self.idx = Idx::Child(i, sub_i);
        }
    }

    fn prev(&mut self, collections: &[CollectionsModel]) {
        match self.idx {
            Idx::None => {}
            _ => {
                let previous_idx = self.idx.clone();
                self.prev_request(collections);

                if self.idx == previous_idx {
                    self.prev_collection();
                }
            }
        }
    }
}

impl CollectionsView {
    pub fn set_render_area(&mut self, area: Rect) {
        self.render_area = area;
    }

    pub fn draw<'painter, 'this: 'painter, 'state: 'painter>(
        &'this self,
        painter: &mut Painter<'painter>,
        state: &'state PaneStore,
    ) {
        painter.render(move |frame| {
            let is_focus = state.is_focus(super::state::ElementFocus::Collections);
            let items: Vec<Item<'_>> = state
                .collections()
                .iter()
                .enumerate()
                .map(|(i, coll)| {
                    let mut itm = Item::new(coll.name());

                    for (sub_i, req) in coll.requests().iter().enumerate() {
                        let name = match state.is_sending_request(SendRequestId(i, sub_i)) {
                            true => format!("pending {}", req.name()),
                            false => req.name().to_string(),
                        };
                        itm.add_child(Item::new(name));
                    }

                    itm
                })
                .collect();

            let collections = CollectionList::default()
                .set_items(items)
                .set_block(
                    Block::bordered().title(" Collections ").border_style(
                        is_focus
                            .then_some(Style::default().blue())
                            .unwrap_or_default(),
                    ),
                )
                .set_openeds(self.openeds.clone())
                .set_idx(self.idx)
                .set_highlight_style(Style::default().green());

            frame.render_widget(collections, self.render_area);
        });

        if self.show_popup {
            painter.render_last(|frame| {
                let area = {
                    let [area] = Layout::vertical([Constraint::Length(3)])
                        .flex(ratatui::layout::Flex::Center)
                        .areas(frame.area());

                    let [area] = Layout::horizontal([Constraint::Percentage(40)])
                        .flex(ratatui::layout::Flex::Center)
                        .areas(area);

                    area
                };

                frame.render_widget(Clear, area);
                self.menu.draw(frame, area);
            });
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent, state: &PaneStore) -> Vec<Action> {
        let mut actions = Vec::new();
        if key.kind == KeyEventKind::Press {
            if self.show_popup {
                match key.code {
                    KeyCode::Enter => {
                        let text = self.menu.text();

                        match self.menu.method_type() {
                            UpsertMethod::CreateRequest => {
                                actions.push(Action::CreateRequest {
                                    coll_idx: self.idx.parent_idx(),
                                    name: text,
                                });
                            }
                            UpsertMethod::CreateCollection => {
                                actions.push(Action::CreateCollection(text));
                                if self.idx == Idx::None {
                                    self.idx = Idx::Parent(0);
                                }
                                self.openeds.insert(state.collections().len());
                            }
                            UpsertMethod::EditRequest => {
                                actions.push(Action::EditRequestName {
                                    idx: self.idx.child_idx(),
                                    name: text,
                                });
                            }
                            UpsertMethod::EditCollection => {
                                actions.push(Action::EditCollectionName {
                                    idx: self.idx.parent_idx(),
                                    new_name: text,
                                });
                            }
                        }

                        self.show_popup = false;
                    }
                    KeyCode::Esc => {
                        self.show_popup = false;
                    }
                    _ => {
                        self.menu.handle_input(Input::from(key));
                    }
                }
            } else {
                match key.code {
                    KeyCode::Tab => {
                        actions.push(Action::NextFocus);
                    }
                    // KeyCode::Enter => match state.idx() {
                    //     Idx::Child(i, sub_i) => mutator.add(SetRquestIdx::new(Some((i, sub_i)))),
                    //     _ => {}
                    // },
                    KeyCode::BackTab => {
                        actions.push(Action::PreviousFocus);
                    }
                    KeyCode::Left | KeyCode::Char('h') => {
                        if let Idx::Parent(i) = self.idx {
                            self.openeds.remove(&i);
                        }
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        if let Idx::Parent(i) = self.idx {
                            self.openeds.insert(i);
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        self.next(state.collections());
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        self.prev(state.collections());
                    }
                    KeyCode::Char('d') | KeyCode::Delete => match self.idx {
                        state::Idx::None => {}
                        state::Idx::Parent(i) => {
                            actions.push(Action::DeleteCollection { idx: i });
                            self.openeds.remove(&i);

                            // So I need set the idx on self!
                            let collections = state.collections();

                            if i == collections.len() - 1 {
                                if i == 0 {
                                    self.idx = Idx::None;
                                } else {
                                    self.idx = Idx::Parent(i - 1);
                                }
                            }
                        }
                        state::Idx::Child(i, sub_i) => {
                            actions.push(Action::DeleteRequest((i, sub_i)));
                            let requests_len = state.collections()[i].requests.len();

                            if sub_i == requests_len - 1 {
                                if sub_i == 0 {
                                    self.idx = Idx::Parent(i);
                                } else {
                                    self.idx = Idx::Child(i, sub_i - 1);
                                }
                            }
                        }
                    },
                    KeyCode::Char('c') => {
                        self.show_popup = true;
                        self.menu.set_state(UpsertMethod::CreateCollection, "");
                    }
                    KeyCode::Char('r') => {
                        if !self.idx.is_none() {
                            self.show_popup = true;
                            self.menu.set_state(UpsertMethod::CreateRequest, "");
                        }
                    }
                    KeyCode::Char('e') => match self.idx {
                        state::Idx::None => {}
                        state::Idx::Parent(i) => {
                            let collection = state.collections().get(i).unwrap();
                            self.menu
                                .set_state(UpsertMethod::EditCollection, &collection.name);
                            self.show_popup = true;
                        }
                        state::Idx::Child(_, _) => {
                            let name = state.current_request().unwrap().name();
                            self.menu.set_state(UpsertMethod::EditRequest, name);
                            self.show_popup = true;
                        }
                    },
                    _ => {}
                }
            }
        }

        actions
    }
}
