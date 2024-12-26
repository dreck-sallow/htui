use ratatui::{
    layout::Rect,
    widgets::{Block, Borders},
    Frame,
};

use crate::tools::tui::core::elements::nested_list::{
    cursor::NestedCursor,
    ui::{NestedListItemUi, NestedListSubItemUi, NestedListUi},
};

use super::state::CollectionState;

pub struct CollectionsUi {}

impl CollectionsUi {
    pub fn render(frame: &mut Frame, area: Rect, state: &CollectionState) {
        let items: Vec<NestedListItemUi<'_>> = state
            .list
            .items
            .iter()
            .map(|itm| {
                // println!("sub: {}", itm.sub_items.len());
                NestedListItemUi::new(itm.inner.name.clone()).sub_items(
                    itm.sub_items
                        .iter()
                        .map(|sub_itm| NestedListSubItemUi::new(sub_itm.name.clone()))
                        .collect(),
                )
            })
            .collect();

        let nested_list = NestedListUi::new(items)
            .with_block(
                Block::default()
                    .title(" Collections  title ")
                    .borders(Borders::ALL),
            )
            // .with_cursor(state.list.get_cursor().clone());
            .with_cursor(NestedCursor::from(0));

        frame.render_widget(nested_list, area);
    }
}
