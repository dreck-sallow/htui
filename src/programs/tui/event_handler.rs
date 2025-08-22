use std::fmt::Debug;

use crossterm::event::KeyEvent;
use tokio::sync::mpsc;

pub type EventSender = mpsc::Sender<AppMessage>;

pub trait EventSource {
    type Message;
    fn run(&mut self, sender: mpsc::Sender<Self::Message>);

    fn end(&mut self);
}

/// Event sending for the app
#[derive(Debug)]
pub enum AppMessage {
    Draw,
    Input(KeyEvent),
    Quit,
}

// #[derive(Debug)]
pub struct Events<M> {
    event_sender: mpsc::Sender<M>,
    event_receiver: mpsc::Receiver<M>,
    sources: Vec<Box<dyn EventSource<Message = M> + 'static>>,
}

impl<M> Events<M> {
    // pub fn new() -> Self {
    //     let (event_sender, event_receiver) = mpsc::channel::<M>(10);

    //     Self {
    //         event_sender,
    //         event_receiver,
    //         sources: Vec::new(),
    //     }
    // }

    pub fn from_sources(_sources: Vec<Box<dyn EventSource<Message = M>>>) -> Self {
        let (event_sender, event_receiver) = mpsc::channel::<M>(100);

        Self {
            event_sender,
            event_receiver,
            sources: _sources.into(),
        }
    }

    // pub fn add_source<S: EventSource<Message = M> + 'static>(&mut self, source: S) {
    //     self.sources.push(Box::new(source));
    // }

    pub fn run(&mut self) {
        for source in &mut self.sources {
            source.run(self.event_sender.clone());
        }
    }

    pub fn stop(&mut self) {
        for source in &mut self.sources {
            source.end();
        }
    }

    pub fn messages_count(&self) -> usize {
        self.event_receiver.len()
    }

    pub fn sender(&self) -> mpsc::Sender<M> {
        self.event_sender.clone()
    }

    pub async fn next_message(&mut self) -> Option<M> {
        self.event_receiver.recv().await
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use tokio::sync::{broadcast, mpsc};

    use super::{EventSource, Events};

    #[derive(Debug)]
    enum Message {
        Delayed,
    }

    struct Delay {
        signal_sender: broadcast::Sender<()>,
        seconds: u8,
    }

    impl Delay {
        pub fn new(secs: u8) -> Self {
            let (tx, _) = tokio::sync::broadcast::channel::<()>(1);
            Self {
                seconds: secs,
                signal_sender: tx,
            }
        }
    }

    impl EventSource for Delay {
        type Message = Message;

        fn run(&mut self, sender: mpsc::Sender<Self::Message>) {
            let seconds = self.seconds;

            let mut rx = self.signal_sender.subscribe();
            tokio::spawn(async move {
                loop {
                    tokio::select! {
                        _ = tokio::time::sleep(Duration::from_secs(seconds as u64)) => {
                            let _ = sender.send(Message::Delayed);
                        }
                        _ = rx.recv() => {
                            break;
                        }
                    }
                }
            });
        }

        fn end(&mut self) {
            let _ = self.signal_sender.send(());
        }
    }

    #[tokio::test]
    async fn test_events() {
        let mut events =
            Events::from_sources(vec![Box::new(Delay::new(4)), Box::new(Delay::new(2))]);

        tokio::time::sleep(Duration::from_secs(6)).await;
        assert_eq!(events.messages_count(), 3); // 2 (4) + 5 (2)
        events.stop();

        tokio::time::sleep(Duration::from_secs(3)).await;
        assert_eq!(events.messages_count(), 3); // same events length

        events.run();
        tokio::time::sleep(Duration::from_secs(5)).await;
        assert_eq!(events.messages_count(), 6);

        // events.finish();
    }
}
