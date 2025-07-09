use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, Frame};

/// Collect render functions for execute after call draw() methods
/// It allow render last (rewrites the frame ui buffer), useful for
/// overlays like dropdowns, menus, popups
pub struct Painter<'a> {
    renders: Vec<Box<dyn FnMut(&mut Frame) + 'a>>,
    last_renders: Vec<Box<dyn FnMut(&mut Frame) + 'a>>,
}

impl<'a> Painter<'a> {
    pub fn new() -> Self {
        Self {
            renders: Vec::new(),
            last_renders: Vec::new(),
        }
    }

    pub fn render<F: FnMut(&mut Frame) + 'a>(&mut self, f: F) {
        self.renders.push(Box::new(f));
    }

    pub fn render_last<F: FnMut(&mut Frame) + 'a>(&mut self, f: F) {
        self.last_renders.push(Box::new(f));
    }

    pub fn draw(mut self, frame: &mut Frame) {
        for render in &mut self.renders {
            render(frame);
        }

        for last_render in &mut self.last_renders {
            last_render(frame);
        }
    }
}

pub trait Drawable<'a, 'painter_fn> {
    type State;
    fn set_render_area(&mut self, _area: Rect) {}
    fn draw<'b: 'painter_fn>(&'a self, painter: &mut Painter<'painter_fn>, state: &'b Self::State);
}

pub trait Interactive<'a: 'painter_fn, 'painter_fn>: Drawable<'a, 'painter_fn> {
    type Mutator;

    fn on_key(&mut self, key: KeyEvent, mutator: &mut Self::Mutator, _state: &Self::State);
    fn on_change_state(&mut self, _state: &Self::State) {}
}

pub trait InteractiveV2 {
    type State;
    type Mutator;

    fn on_key(&mut self, key: KeyEvent, mutator: &mut Self::Mutator, _state: &Self::State);
    fn on_change_state(&mut self, _state: &Self::State) {}
}

// pub trait Drawable<'a: 'painter_fn, 'painter_fn> {
//     type State;
//     fn set_render_area(&mut self, _area: Rect) {}
//     fn draw(&'a self, painter: &mut Painter<'painter_fn>, state: &'a Self::State);
// }

// pub trait Interactive<'a: 'painter_fn, 'painter_fn>: Drawable<'a, 'painter_fn> {
//     type Mutator;

//     fn on_key(&mut self, key: KeyEvent, mutator: &mut Self::Mutator, _state: &Self::State);
//     fn on_change_state(&mut self, _state: &Self::State) {}
// }
