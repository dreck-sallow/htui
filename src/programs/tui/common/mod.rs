use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, Frame};

pub mod action_history;
pub mod component;

pub type IsFocused = bool;

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

    pub fn clamp_index(current: Option<usize>, len: usize) -> Option<usize> {
        match current {
            Some(idx) => {
                if len == 0 {
                    return None;
                } else if idx >= len {
                    return Some(idx - 1);
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

    pub fn delete_element<E>(current_idx: Option<usize>, list: &mut Vec<E>) -> Option<usize> {
        match current_idx {
            Some(idx) => {
                list.remove(idx);
                self::clamp_index(current_idx, list.len())
            }
            None => None,
        }
    }
}

pub trait UiComposedElement<'params> {
    type Params: 'params;

    fn set_area(&mut self, area: Rect, viewport_area: Rect);

    fn draw(&self, params: Self::Params, frame: &mut Frame);

    fn draw_overlay(&self, _params: Self::Params, _frame: &mut Frame) {
        unimplemented!()
    }
}

pub trait UiElementV2<'params> {
    type Params: 'params;

    fn draw(&self, params: Self::Params, frame: &mut Frame);
}

pub trait Interactive<'params> {
    type Effect;
    type Params: 'params;

    fn handle_key(&mut self, params: Self::Params, key: KeyEvent) -> Self::Effect;

    fn is_input_focus(&self) -> bool {
        false
    }

    fn is_visible_overlay(&self) -> bool {
        false
    }
}
