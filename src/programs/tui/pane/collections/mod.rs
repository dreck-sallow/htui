use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use list::{CollectionList, Item};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, Clear},
};
use tui_textarea::Input;
use upsert_item::{UpsertItemPopup, UpsertMethod};

use crate::{
    programs::tui::element_view::Painter,
    store::models::{CollectionsModel, RequestModel, SendRequestId},
};

use super::{
    mutation_history::MutationCollector,
    mutations::{
        CreateCollection, CreateRequest, EditCollectionName, OpenCloseCollection, RemoveCollection,
        RemoveRequest, SetFocus, SetIdx,
    },
    state::PaneState,
};

mod list;
pub mod state;
mod upsert_item;

pub struct CollectionsView {
    render_area: Rect,
    menu: UpsertItemPopup,
    show_popup: bool,
}

impl CollectionsView {
    pub fn new() -> Self {
        Self {
            render_area: Rect::default(),
            menu: UpsertItemPopup::new(),
            show_popup: false,
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
        state: &'state PaneState,
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
                .set_openeds(state.opened_collections())
                .set_idx(state.idx())
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

    pub fn handle_key(
        &mut self,
        key: KeyEvent,
        collector: &mut MutationCollector,
        state: &PaneState,
    ) {
        if key.kind == KeyEventKind::Press {
            if self.show_popup {
                match key.code {
                    KeyCode::Enter => {
                        // let idx = state.idx();
                        let text = self.menu.text();

                        match self.menu.method_type() {
                            UpsertMethod::CreateRequest => {
                                let req = RequestModel::new(text);
                                collector.add(CreateRequest::new(state.idx().parent_idx(), req));
                            }
                            UpsertMethod::CreateCollection => {
                                let coll = CollectionsModel::new(text);
                                collector.add(CreateCollection::new(coll));
                            }
                            UpsertMethod::EditRequest => {
                                // mutator.add(EditRequest::new(
                                //     idx.child_idx(),
                                //     RequestEditType::Name(text),
                                // ));
                            }
                            UpsertMethod::EditCollection => {
                                collector
                                    .add(EditCollectionName::new(state.idx().parent_idx(), text));
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
                        collector.add(SetFocus::for_next());
                    }
                    // KeyCode::Enter => match state.idx() {
                    //     Idx::Child(i, sub_i) => mutator.add(SetRquestIdx::new(Some((i, sub_i)))),
                    //     _ => {}
                    // },
                    KeyCode::BackTab => {
                        collector.add(SetFocus::for_previous());
                    }
                    KeyCode::Left | KeyCode::Char('h') => {
                        collector.add(OpenCloseCollection::for_close());
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        collector.add(OpenCloseCollection::for_open());
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        collector.add(SetIdx::for_next());
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        collector.add(SetIdx::for_previous());
                    }
                    KeyCode::Char('d') | KeyCode::Delete => match state.idx() {
                        state::Idx::None => {}
                        state::Idx::Parent(i) => {
                            collector.add(RemoveCollection::new(i));
                        }
                        state::Idx::Child(i, sub_i) => {
                            collector.add(RemoveRequest::new((i, sub_i)));
                        }
                    },
                    KeyCode::Char('c') => {
                        self.show_popup = true;
                        self.menu.set_state(UpsertMethod::CreateCollection, "");
                    }
                    KeyCode::Char('r') => {
                        if !state.idx().is_none() {
                            self.show_popup = true;
                            self.menu.set_state(UpsertMethod::CreateRequest, "");
                        }
                    }
                    KeyCode::Char('e') => match state.idx() {
                        state::Idx::None => {}
                        state::Idx::Parent(_) => {
                            let name = state.current_collection().unwrap().name();
                            self.menu.set_state(UpsertMethod::EditCollection, name);
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
    }
}
