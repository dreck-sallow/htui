use std::{io, time::Duration};

use ratatui::crossterm;
use tokio::{sync::mpsc, task::JoinHandle};

pub enum Event {
    Quit,
    Key(crossterm::event::KeyEvent),
}

pub struct EventHandler {
    rx: mpsc::UnboundedReceiver<Event>,
    task: JoinHandle<io::Result<()>>,
}

impl EventHandler {
    pub fn new(time_rate: u64) -> Self {
        let tick = Duration::from_millis(time_rate);

        let (tx, mut rx) = mpsc::unbounded_channel();

        let task = tokio::spawn(async move {
            loop {
                if crossterm::event::poll(tick)? {
                    match crossterm::event::read()? {
                        crossterm::event::Event::Key(e) => {
                            tx.send(Event::Key(e));
                        }
                        _ => unimplemented!(),
                    }
                }
            }
            Ok::<(), io::Error>(())
        });

        EventHandler { rx, task }
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.rx.recv().await
    }

    pub fn stop(&self) {
        self.task.abort()
    }
}
