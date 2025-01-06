mod state;
// mod state_handler;
// mod ui;
// pub use ui::CollectionsUi;

use crossterm::event::KeyEvent;
use ratatui::{
    layout::Rect,
    widgets::{Block, Borders},
    Frame,
};

pub use state::*;

use crate::tools::tui::core::elements::nested_list::ui_v2::{NestedList, NestedListItem};

pub struct Collections;

impl Collections {
    pub fn render(frame: &mut Frame, area: Rect, state: &CollectionState) {
        let items: Vec<NestedListItem> = state
            .list
            .items_ref()
            .iter()
            .flat_map(|item| {
                let parent = NestedListItem::L1 {
                    text: item.inner.name.clone().into(),
                };

                let mut list = vec![parent];

                list.append(
                    &mut item
                        .sub_items
                        .iter()
                        .map(|sub_item| NestedListItem::L2 {
                            text: sub_item.name.clone().into(),
                        })
                        .collect(),
                );

                list
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

    pub fn event(state: &mut CollectionState, key: &KeyEvent) {
        match key.code {
            crossterm::event::KeyCode::Left => {}
            crossterm::event::KeyCode::Right => {}
            crossterm::event::KeyCode::Up => {
                state.list.prev();
            }
            crossterm::event::KeyCode::Down => {
                state.list.next();
            }
            _ => {}
        }
    }
}
