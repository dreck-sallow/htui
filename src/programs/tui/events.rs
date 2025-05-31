use crossterm::event::KeyEvent;
use tokio::sync::mpsc;

pub trait Source {
    fn register_sender(&mut self, sender: mpsc::UnboundedSender<Event>);
    fn start_process(&mut self);
}

pub type EventSender = mpsc::UnboundedSender<Event>;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Event {
    Draw,
    Input(KeyEvent),
    KeyBinding(KeyEvent, KeyEvent),
    Quit,
}

pub struct Events<'a> {
    tx: EventSender,
    rx: mpsc::UnboundedReceiver<Event>,
    sources: Vec<Box<dyn Source + 'a>>,
    _started_process: bool,
}

impl<'a> Events<'a> {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            tx,
            rx,
            sources: Vec::new(),
            _started_process: false,
        }
    }

    pub fn add_source<S: Source + 'a>(&mut self, mut source: S) {
        // TODO: start the process on new pushed sources
        assert!(
            !self._started_process,
            "Cannot push a source when is listen"
        );
        source.register_sender(self.tx.clone());
        self.sources.push(Box::new(source));
    }

    pub fn listen(&mut self) {
        for source in &mut self.sources {
            source.start_process();
        }
    }

    pub fn sender(&self) -> mpsc::UnboundedSender<Event> {
        self.tx.clone()
    }

    pub async fn next_event(&mut self) -> Option<Event> {
        self.rx.recv().await
    }
}
