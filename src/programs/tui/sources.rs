use crossterm::event::{
    Event as TerminalEvent, EventStream, KeyCode, KeyEvent, KeyEventKind, KeyModifiers,
};
use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;

use super::events::{Event, Source};

#[derive(Default)]
pub struct TerminalSource {
    tx: Option<mpsc::UnboundedSender<Event>>,
}

impl Source for TerminalSource {
    fn register_sender(&mut self, sender: mpsc::UnboundedSender<super::events::Event>) {
        self.tx = Some(sender);
    }

    fn start_process(&mut self) {
        if let Some(tx) = self.tx.take() {
            let _ = tokio::spawn(async move {
                let mut stream = EventStream::new();
                while let Some(Ok(ev)) = stream.next().fuse().await {
                    match ev {
                        TerminalEvent::Key(key_event) => {
                            let event = if let KeyEvent {
                                code: KeyCode::Char('c'),
                                kind: KeyEventKind::Press,
                                modifiers: KeyModifiers::CONTROL,
                                ..
                            } = key_event
                            {
                                Event::Quit
                            } else {
                                Event::Input(key_event)
                            };

                            if tx.send(event).is_err() {
                                break;
                            }
                        }

                        _ => {}
                    }
                }
            });
        }
    }
}
