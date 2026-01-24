use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, Frame};

use crate::programs::tui_v2::{
    common::{elements::ui_block, input::input_mode::InputMode},
    pane::state::PaneState,
};

pub enum UpsertAction {
    CreateCollection,
    CreateRequest,
    EditCollection,
    EditRequest,
}

pub struct UpertItemPopup {
    action: UpsertAction,
    input: InputMode,
}

impl UpertItemPopup {
    fn title(&self) -> &'static str {
        match self.action {
            UpsertAction::CreateCollection => " Create Collection ",
            UpsertAction::CreateRequest => " Create Request ",
            UpsertAction::EditCollection => " Edit Collection ",
            UpsertAction::EditRequest => " Edit Request ",
        }
    }

    pub fn draw(&self, area: Rect, frame: &mut Frame, state: &PaneState) {
        let block = ui_block(self.title(), true);
        let inner_area = block.inner(area);

        frame.render_widget(block, area);
        self.input.draw(inner_area, frame);
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            // crossterm::event::KeyCode::Enter if !self.input.is_editing() => {}
            crossterm::event::KeyCode::Esc => todo!(),
            _ => {
                self.input.handle_key(key);
            }
        }
    }
}
