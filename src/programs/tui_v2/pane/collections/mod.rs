use ratatui::{layout::Rect, Frame};

use crate::programs::tui_v2::common::elements::ui_block;

use super::state::PaneState;
mod list;

pub struct CollectionsSidebar;

impl CollectionsSidebar {
    pub fn draw(&self, area: Rect, frame: &mut Frame, state: &PaneState) {
        let block = ui_block(
            format!(" Collections ({}) ", state.collections.size()),
            false,
        );

        let inner_area = block.inner(area);

        frame.render_widget(block, area);
    }
}
