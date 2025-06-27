use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, Frame};

pub trait ElementView {
    type State;

    fn set_area(&mut self, _area: Rect) {}
    fn draw(&self, frame: &mut Frame, state: &Self::State);
    fn on_key(&mut self, _key: KeyEvent, _state: &mut Self::State) {}
    // fn on_resize(&mut self, _area: Rect, _state: &mut Self::State) {}
    fn on_change_state(&mut self, _state: &Self::State) {}
}
