mod state;

use crossterm::event::KeyEvent;
use ratatui::{
    layout::Rect,
    widgets::{Block, Borders},
    Frame,
};

pub use state::*;

use crate::tools::tui::{
    core::elements::nested_list::item_v2::NestedListItem as NestedLisItemType, element::Element,
};
use crate::tools::tui::{
    core::elements::nested_list::{
        ui_v2::{NestedList, NestedListItem},
        NestedListITemState,
    },
    element::EffectCommand,
};

pub struct Collections {
    show_popup: bool,
}

impl Collections {
    pub fn new() -> Self {
        Self { show_popup: false }
    }
}

impl Element for Collections {
    type State = CollectionState;

    fn render(&mut self, frame: &mut Frame, area: Rect, state: &Self::State) {
        let items: Vec<NestedListItem> = state
            .list
            .items()
            .iter()
            .flat_map(|item| match item {
                NestedLisItemType::Single(single) => {
                    vec![NestedListItem::L1 {
                        text: single.name.clone().into(),
                    }]
                }
                NestedLisItemType::Group { inner, items } => {
                    let mut list = vec![NestedListItem::L1 {
                        text: inner.name.clone().into(),
                    }];

                    list.append(
                        &mut items
                            .iter()
                            .map(|sub_item| {
                                if let NestedLisItemType::Single(single) = sub_item {
                                    NestedListItem::L2 {
                                        text: single.name.clone().into(),
                                    }
                                } else {
                                    unreachable!()
                                }
                            })
                            .collect(),
                    );

                    list
                } // NestedLisItemType::Single(single) => {
                  //     vec![NestedListItem::L1 {
                  //         text: single.0.name.clone().into(),
                  //     }]
                  // }
                  // NestedLisItemType::Group(multiple) => {
                  //     let mut list = vec![NestedListItem::L1 {
                  //         text: multiple.inner().name.clone().into(),
                  //     }];

                  //     list.append(
                  //         &mut multiple
                  //             .sub_items()
                  //             .iter()
                  //             .map(|sub_item| NestedListItem::L2 {
                  //                 text: sub_item.0.name.clone().into(),
                  //             })
                  //             .collect(),
                  //     );

                  //     list
                  // }
            })
            .collect();

        let nested_list = NestedList::new(items)
            .with_block(
                Block::default()
                    .title(" Collections ")
                    .borders(Borders::ALL),
            )
            .with_cursor(state.cursor());

        frame.render_widget(nested_list, area);
    }

    fn event(
        &mut self,
        key: &KeyEvent,
        state: &mut Self::State,
    ) -> crate::tools::tui::element::EffectCommand {
        match key.code {
            crossterm::event::KeyCode::Left => {}
            crossterm::event::KeyCode::Right => {}
            crossterm::event::KeyCode::Up => {
                state.prev();
            }
            crossterm::event::KeyCode::Down => {
                state.next();
            }
            crossterm::event::KeyCode::Char('d') => {
                state.list.remove();
            }
            crossterm::event::KeyCode::Char('c') => {
                state.clone_item();
            }
            _ => {}
        }

        // state
        if let Some(itm) = state.list.current_inner() {
            return match itm {
                crate::tools::tui::core::elements::nested_list::item_v2::NestedListItemState::Single(request) => EffectCommand::SetRequest {
                    method: request.method.clone(),
                    url: request.url.clone(),
                },
                crate::tools::tui::core::elements::nested_list::item_v2::NestedListItemState::Group(_) => EffectCommand::SetRequest {
                    method: "GET".into(),
                    url: "http://localhost:3000".into(),
                },
            };
        }

        EffectCommand::Nothing
    }
}
