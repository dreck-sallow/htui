use crossterm::event::KeyEvent;

use super::state::CollectionState;

pub struct CollectionsHandler;

impl CollectionsHandler {
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
