use ratatui::{layout::Rect, Frame};

use crate::programs::tui_v2::pane::state::PaneState;

pub enum UpsertAction {
    CreateCollection,
    CreateRequest,
    EditCollection,
    EditRequest,
}

pub struct UpertItemPopup {
    action: UpsertAction,
}

impl UpertItemPopup {
    pub fn draw(&self, area: Rect, frame: &mut Frame, state: &PaneState) {}
}
