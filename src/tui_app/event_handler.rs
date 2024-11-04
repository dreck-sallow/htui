use crossterm::event::KeyEventKind;
use tokio::{
    sync::{
        mpsc::{self, UnboundedReceiver},
        oneshot,
    },
    task::JoinHandle,
    time,
};

use futures::{FutureExt, StreamExt};

#[derive(Debug)]
pub enum AppEvent {
    Key(crossterm::event::KeyEvent),
    AppTick,
    Error,
}

pub struct AppEventHandler {
    task: Option<JoinHandle<()>>,
    rx: UnboundedReceiver<AppEvent>,
    cancel_sender: tokio::sync::watch::Sender<bool>,
}

impl AppEventHandler {
    pub fn new(tick_rate: u64) -> Self {
        let tick_rate = time::Duration::from_millis(tick_rate);

        let (cancel_x, mut cancel_r) = tokio::sync::watch::channel(false);

        let (tx, rx) = mpsc::unbounded_channel::<AppEvent>();

        let _tx = tx.clone();

        let task = tokio::spawn(async move {
            let mut interval = time::interval(tick_rate);
            let mut event_reader = crossterm::event::EventStream::new();
            loop {
                let delay = interval.tick();
                let event = event_reader.next().fuse();

                tokio::select! {
                    _ = cancel_r.changed() => {
                        if *cancel_r.borrow() {
                            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                            break;
                        }
                    },
                    maybe_event = event => {
                        if let Some(opt_ev) = maybe_event {
                            match opt_ev {
                                Ok(ev) => {
                                    if let crossterm::event::Event::Key(key) = ev {
                                        if key.kind == KeyEventKind::Press {
                                            tx.send(AppEvent::Key(key)).unwrap();
                                        }
                                    }
                                },
                                Err(err) => {
                                    tx.send(AppEvent::Error).unwrap();
                                }
                            }

                        }
                    },
                    _ = delay => {
                      _tx.send(AppEvent::AppTick).unwrap();
                    }
                };
            }
        });

        Self {
            task: Some(task),
            rx,
            cancel_sender: cancel_x,
        }
    }

    pub async fn next(&mut self) -> Option<AppEvent> {
        self.rx.recv().await
    }

    pub async fn stop(&mut self) {
        self.cancel_sender.send(true).unwrap();
        if let Some(handle) = self.task.take() {
            handle.await.unwrap()
        }
    }
}
