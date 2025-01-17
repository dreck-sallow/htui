use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, Frame};

/// Command used for side effects for
/// Other things, primarily used for change
/// global state
pub enum EffectCommand {
    SetRequest {
        method: String,
        url: String,
    },
    /// Result used for no effect
    Nothing,
}

pub trait Element {
    type State;
    /// TODO: I'm use mutable self for render because stateful list state
    fn render(&mut self, frame: &mut Frame, area: Rect, state: &Self::State);
    fn event(&mut self, _key: &KeyEvent, _state: &mut Self::State) -> EffectCommand {
        EffectCommand::Nothing
    }
}
