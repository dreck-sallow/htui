use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, Frame};

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

pub trait Drawable {
    type Params;

    fn draw<'a: 'painter, 'painter>(
        &'a self,
        painter: &mut Painter<'painter>,
        params: Self::Params,
    );
    fn set_area(&mut self, _area: Rect) {}
}

pub trait Interactive {
    type Effect;
    type Params;
    fn on_key(&mut self, key: KeyEvent, params: Self::Params) -> Option<Self::Effect>;
}

pub trait WithHistory {
    fn undo(&mut self);
    fn redo(&mut self);
}
