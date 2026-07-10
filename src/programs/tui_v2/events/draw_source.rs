use std::time::Duration;

use tokio::{
    sync::watch::{self, Sender},
    task::JoinHandle,
    time::sleep,
};

use super::{EventMsg, EventSource};

pub struct DrawSource {
    task: Option<JoinHandle<()>>,
    notify_sender: Sender<()>,
}

impl DrawSource {
    pub fn new() -> Self {
        let (tx, _) = watch::channel(());

        Self {
            task: None,
            notify_sender: tx,
        }
    }

    pub fn signal(&self) -> DrawSignal {
        DrawSignal {
            notify_sender: self.notify_sender.clone(),
        }
    }
}

impl EventSource for DrawSource {
    fn start(&mut self, sender: tokio::sync::mpsc::Sender<EventMsg>) {
        let mut receiver = self.notify_sender.subscribe();

        let jh = tokio::spawn(async move {
            // First draw!
            if sender.send(EventMsg::Draw).await.is_err() {
                return;
            }

            while let Ok(()) = receiver.changed().await {
                sleep(Duration::from_millis(100)).await;
                if sender.send(EventMsg::Draw).await.is_err() {
                    break;
                }
            }
        });

        self.task = Some(jh);
    }

    fn finish(&mut self) {
        if let Some(task) = self.task.take() {
            task.abort();
        }
    }
}

#[derive(Clone)]
pub struct DrawSignal {
    notify_sender: Sender<()>,
}

impl DrawSignal {
    pub fn new(sender: Sender<()>) -> Self {
        Self {
            notify_sender: sender,
        }
    }

    pub fn draw(&self) {
        let _ = self.notify_sender.send(());
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::time::sleep;

    use crate::programs::tui_v2::events::{draw_source::DrawSource, EventSource};

    #[tokio::test(flavor = "multi_thread")]
    pub async fn test_draw() {
        let (tx, rx) = tokio::sync::mpsc::channel(10);

        let mut draw_source = DrawSource::new();
        let draw_signal = draw_source.signal();

        draw_source.start(tx);

        draw_signal.draw();
        draw_signal.draw();
        draw_signal.draw();
        draw_signal.draw();
        draw_signal.draw();

        assert_eq!(rx.len(), 0);

        sleep(Duration::from_millis(100)).await;
        assert_eq!(rx.len(), 1);

        sleep(Duration::from_millis(200)).await;
        assert_eq!(rx.len(), 1);

        draw_signal.draw();
        draw_signal.draw();

        sleep(Duration::from_millis(100)).await;
        assert_eq!(rx.len(), 2);

        draw_source.finish();
    }
}
