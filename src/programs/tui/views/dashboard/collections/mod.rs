use collections::{Collections, Item};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, Clear},
};
use tui_textarea::Input;
use upsert_item_menu::{UpsertItemMenu, UpsertMethod};

use crate::{
    programs::tui::element_view::{Drawable, InteractiveV2, Painter},
    store::models::{CollectionsModel, ProjectModel, RequestModel, SendRequestId},
};

use super::pane_state::{
    history::MutationCollector,
    mutations::{
        AddCollection, AddRequest, EditCollectionName, EditRequest, FocusNavigation,
        RemoveCollection, RemoveRequest, RequestEditType, SetFocus, SetRquestIdx,
    },
    PaneState,
};

mod collections;
mod state;
mod upsert_item_menu;

pub use state::{CollectionsState, Idx};

pub struct CollectionsComponent {
    render_area: Rect,
    state: CollectionsState<String>,
    menu: UpsertItemMenu,
    show_popup: bool,
}

impl CollectionsComponent {
    pub fn new(project: &ProjectModel) -> Self {
        let mut collections_state = CollectionsState::new();

        for collection in project.collections() {
            let children = collection
                .requests()
                .iter()
                .map(|req| req.id().to_string())
                .collect();

            collections_state.add_collection((collection.id().to_string(), children));
        }

        Self {
            render_area: Rect::default(),
            state: collections_state,
            menu: UpsertItemMenu::new(),
            show_popup: false,
        }
    }
}

impl<'a: 'painter_fn, 'painter_fn> Drawable<'a, 'painter_fn> for CollectionsComponent {
    type State = PaneState;

    fn draw<'b: 'painter_fn>(&'a self, painter: &mut Painter<'painter_fn>, state: &'b Self::State) {
        painter.render(move |frame| {
            let reader = state.reader();
            let is_focus = state.is_focused(super::focus::ElementFocus::Collections);
            let items: Vec<Item<'_>> = reader
                .collections()
                .iter()
                .enumerate()
                .map(|(i, coll)| {
                    let mut itm = Item::new(coll.name());

                    for (sub_i, req) in coll.requests().iter().enumerate() {
                        let name = match reader.is_sending_request(SendRequestId(i, sub_i)) {
                            true => format!("pending {}", req.name()),
                            false => req.name().to_string(),
                        };
                        itm.add_child(Item::new(name));
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

    fn set_render_area(&mut self, _area: Rect) {
        self.render_area = _area;
    }
}

// impl<'a: 'painter_fn, 'painter_fn> Interactive<'a, 'painter_fn> for CollectionsComponent {
//     type Mutator = MutationCollector<'a>;

//     fn on_key(&mut self, key: KeyEvent, mutator: &mut Self::Mutator, state: &Self::State) {
//         if key.kind == KeyEventKind::Press {
//             if self.show_popup {
//                 match key.code {
//                     KeyCode::Enter => {
//                         let idx = self.state.idx();
//                         let text = self.menu.text();

//                         match self.menu.method_type() {
//                             UpsertMethod::CreateRequest => {
//                                 mutator.add(AddRequest::new(
//                                     idx.parent_idx(),
//                                     RequestModel::new(text),
//                                 ));
//                             }
//                             UpsertMethod::CreateCollection => {
//                                 mutator.add(AddCollection::new(CollectionsModel::new(text)));
//                             }
//                             UpsertMethod::EditRequest => {
//                                 mutator.add(EditRequest::new(
//                                     idx.child_idx(),
//                                     RequestEditType::Name(text),
//                                 ));
//                             }
//                             UpsertMethod::EditCollection => {
//                                 mutator.add(EditCollectionName::new(text, idx.parent_idx()));
//                             }
//                         }

//                         self.show_popup = false;
//                     }
//                     KeyCode::Esc => {
//                         self.show_popup = false;
//                     }
//                     _ => {
//                         self.menu.handle_input(Input::from(key));
//                     }
//                 }
//             } else {
//                 match key.code {
//                     KeyCode::Tab => {
//                         mutator.add(SetFocus::new(FocusNavigation::Next));
//                     }
//                     KeyCode::Enter => match self.state.idx() {
//                         Idx::Child(i, sub_i) => mutator.add(SetRquestIdx::new(Some((i, sub_i)))),
//                         _ => {}
//                     },
//                     KeyCode::BackTab => {
//                         mutator.add(SetFocus::new(FocusNavigation::Prev));
//                     }
//                     KeyCode::Left | KeyCode::Char('h') => {
//                         self.state.close_collection(true);
//                     }
//                     KeyCode::Right | KeyCode::Char('l') => {
//                         self.state.open_collection(true);
//                     }
//                     KeyCode::Down | KeyCode::Char('j') => {
//                         self.state.next();
//                     }
//                     KeyCode::Up | KeyCode::Char('k') => {
//                         self.state.prev();
//                     }
//                     KeyCode::Char('d') | KeyCode::Delete => {
//                         match self.state.idx() {
//                             state::Idx::None => {}
//                             state::Idx::Parent(i) => {
//                                 // SUGGEST: not delete, or react on on_change_state
//                                 self.state.delete_collection();
//                                 mutator.add(RemoveCollection::new(i));
//                             }
//                             state::Idx::Child(i, sub_i) => {
//                                 // SUGGEST: not delete, or react on on_change_state
//                                 self.state.delete_request();
//                                 mutator.add(RemoveRequest::new((i, sub_i)));
//                             }
//                         }
//                     }
//                     KeyCode::Char('c') => {
//                         self.show_popup = true;
//                         self.menu.set_state(UpsertMethod::CreateCollection, "");
//                     }
//                     KeyCode::Char('r') => {
//                         self.show_popup = true;
//                         if !self.state.idx().is_none() {
//                             self.menu.set_state(UpsertMethod::CreateRequest, "");
//                         }
//                     }
//                     KeyCode::Char('e') => {
//                         let reader = state.reader();

//                         match self.state.idx() {
//                             state::Idx::None => {}
//                             state::Idx::Parent(i) => {
//                                 let name = reader.collections()[i].name();
//                                 self.menu.set_state(UpsertMethod::EditCollection, name);
//                                 self.show_popup = true;
//                             }
//                             state::Idx::Child(i, sub_i) => {
//                                 let name = reader.collections()[i].requests()[sub_i].name();
//                                 self.menu.set_state(UpsertMethod::EditRequest, name);
//                                 self.show_popup = true;
//                             }
//                         }
//                     }
//                     _ => {}
//                 }
//             }
//         }
//     }

//     fn on_change_state(&mut self, state: &Self::State) {
//         let reader = state.reader();

//         for collection in reader.collections() {
//             let children = collection
//                 .requests()
//                 .iter()
//                 .map(|req| req.id().to_string())
//                 .collect();

//             self.state
//                 .add_collection((collection.id().to_string(), children));
//         }
//     }
// }

impl InteractiveV2 for CollectionsComponent {
    type State = PaneState;

    type Mutator = MutationCollector;

    fn on_key(&mut self, key: KeyEvent, mutator: &mut Self::Mutator, state: &Self::State) {
        if key.kind == KeyEventKind::Press {
            if self.show_popup {
                match key.code {
                    KeyCode::Enter => {
                        let idx = self.state.idx();
                        let text = self.menu.text();

                        match self.menu.method_type() {
                            UpsertMethod::CreateRequest => {
                                let req = RequestModel::new(text);
                                let coll_id = state.reader().collections()[idx.parent_idx()]
                                    .id()
                                    .to_string();
                                self.state.add_request(coll_id, req.id().to_string());
                                mutator.add(AddRequest::new(idx.parent_idx(), req));
                            }
                            UpsertMethod::CreateCollection => {
                                let coll = CollectionsModel::new(text);
                                self.state
                                    .add_collection((coll.id().to_string(), Vec::new()));
                                mutator.add(AddCollection::new(coll));
                            }
                            UpsertMethod::EditRequest => {
                                mutator.add(EditRequest::new(
                                    idx.child_idx(),
                                    RequestEditType::Name(text),
                                ));
                            }
                            UpsertMethod::EditCollection => {
                                mutator.add(EditCollectionName::new(text, idx.parent_idx()));
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
                        mutator.add(SetFocus::new(FocusNavigation::Next));
                    }
                    KeyCode::Enter => match self.state.idx() {
                        Idx::Child(i, sub_i) => mutator.add(SetRquestIdx::new(Some((i, sub_i)))),
                        _ => {}
                    },
                    KeyCode::BackTab => {
                        mutator.add(SetFocus::new(FocusNavigation::Prev));
                    }
                    KeyCode::Left | KeyCode::Char('h') => {
                        self.state.close_collection(true);
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        self.state.open_collection(true);
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        self.state.next();
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        self.state.prev();
                    }
                    KeyCode::Char('d') | KeyCode::Delete => {
                        match self.state.idx() {
                            state::Idx::None => {}
                            state::Idx::Parent(i) => {
                                // SUGGEST: not delete, or react on on_change_state
                                self.state.delete_collection();
                                mutator.add(RemoveCollection::new(i));
                            }
                            state::Idx::Child(i, sub_i) => {
                                // SUGGEST: not delete, or react on on_change_state
                                self.state.delete_request();
                                mutator.add(RemoveRequest::new((i, sub_i)));
                            }
                        }
                    }
                    KeyCode::Char('c') => {
                        self.show_popup = true;
                        self.menu.set_state(UpsertMethod::CreateCollection, "");
                    }
                    KeyCode::Char('r') => {
                        self.show_popup = true;
                        if !self.state.idx().is_none() {
                            self.menu.set_state(UpsertMethod::CreateRequest, "");
                        }
                    }
                    KeyCode::Char('e') => {
                        let reader = state.reader();

                        match self.state.idx() {
                            state::Idx::None => {}
                            state::Idx::Parent(i) => {
                                let name = reader.collections()[i].name();
                                self.menu.set_state(UpsertMethod::EditCollection, name);
                                self.show_popup = true;
                            }
                            state::Idx::Child(i, sub_i) => {
                                let name = reader.collections()[i].requests()[sub_i].name();
                                self.menu.set_state(UpsertMethod::EditRequest, name);
                                self.show_popup = true;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // fn on_change_state(&mut self, state: &Self::State) {
    //     let reader = state.reader();

    //     for collection in reader.collections() {
    //         let children = collection
    //             .requests()
    //             .iter()
    //             .map(|req| req.id().to_string())
    //             .collect();

    //         self.state
    //             .add_collection((collection.id().to_string(), children));
    //     }
    // }
}
