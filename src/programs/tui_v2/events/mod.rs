use std::collections::HashMap;

use crossterm::event::KeyEvent;
use draw_source::DrawSource;
use terminal_source::TerminalSource;
use tokio::sync::mpsc::{self, Receiver, Sender};

mod draw_source;
mod terminal_source;

pub use draw_source::DrawSignal;

#[derive(Debug)]
pub enum EventMsg {
    Draw,
    Key(KeyEvent),
    Quit,
}

trait EventSource {
    fn start(&mut self, sender: Sender<EventMsg>);
    fn finish(&mut self);
}

#[derive(Hash, Eq, PartialEq, PartialOrd)]
pub enum SourceId {
    Draw,
    Key,
}

pub struct Events {
    sources: HashMap<SourceId, Box<dyn EventSource>>,
    receiver: Receiver<EventMsg>,
    sender: Sender<EventMsg>,
}

impl Events {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(100);

        Self {
            sources: HashMap::new(),
            receiver: rx,
            sender: tx,
        }
    }

    fn add_source<S: EventSource + 'static>(&mut self, id: SourceId, source: S) {
        self.sources.insert(id, Box::new(source));
    }

    pub fn start(&mut self) {
        for source in self.sources.values_mut() {
            source.start(self.sender.clone());
        }
    }

    pub fn finish(mut self) {
        for source in self.sources.values_mut() {
            source.finish();
        }
    }

    pub async fn next_event(&mut self) -> Option<EventMsg> {
        self.receiver.recv().await
    }
}

pub fn create_events() -> (Events, DrawSignal) {
    let mut events = Events::new();

    let draw_source = DrawSource::new();
    let draw_signal = draw_source.signal();

    events.add_source(SourceId::Draw, draw_source);
    events.add_source(SourceId::Key, TerminalSource::new());

    (events, draw_signal)
}
