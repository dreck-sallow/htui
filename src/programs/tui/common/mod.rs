use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, Frame};

pub mod action_history;
pub mod component;

pub mod input;

/// Common utils for work with lists
pub mod list_utils {
    /// Go to next item index on the list
    pub fn next(current: Option<usize>, len: usize) -> Option<usize> {
        match current {
            Some(idx) => {
                // One fore the last item: so next to last
                if idx < len.saturating_sub(1) {
                    return Some(idx + 1);
                }
            }
            None => {
                if len > 0 {
                    return Some(0);
                }
            }
        }

        current
    }

    /// Go to next previous index on the list
    pub fn prev(current: Option<usize>) -> Option<usize> {
        if let Some(idx) = current {
            if idx > 0 {
                return Some(idx - 1);
            }
        }
        current
    }
}

pub trait UiElement {
    type Params;

    /// Set the render visual area into the element
    fn set_area(&mut self, area: Rect);

    /// Draw the elements in the frame
    fn draw(&self, params: Self::Params, frame: &mut Frame);

    /// Last rendering, used for show overlays
    fn draw_overlay(&self, _params: Self::Params, _frame: &mut Frame) {}
}

pub trait InteractiveElement<'params> {
    type Params: 'params;

    fn handle_key(&mut self, params: Self::Params, key: KeyEvent);

    fn is_editing(&self) -> bool {
        false
    }
}

pub trait InteractiveElementEff<'params> {
    type Effect;
    type Params: 'params;

    fn handle_key(&mut self, params: Self::Params, key: KeyEvent) -> Self::Effect;

    fn is_editing(&self) -> bool {
        false
    }
}
