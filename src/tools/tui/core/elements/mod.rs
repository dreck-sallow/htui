use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, Frame};

pub mod nested_list;

pub trait Element {
    type State;
    fn render(&self, frame: &mut Frame, area: Rect, state: &Self::State);
    fn event(&mut self, _state: &mut Self::State, _key: &KeyEvent) {}
}
