use crossterm::event::{
    Event as TerminalEvent, EventStream, KeyCode, KeyEvent, KeyEventKind, KeyModifiers,
};
use futures::StreamExt;
use tokio::sync::{broadcast, mpsc};

use super::event_handler::{AppMessage, EventSource};

pub struct TerminalSourceV2 {
    signal_sender: broadcast::Sender<()>,
}

impl TerminalSourceV2 {
    pub fn new() -> Self {
        let (tx, _rx) = tokio::sync::broadcast::channel::<()>(1);
        Self { signal_sender: tx }
    }
}

impl EventSource for TerminalSourceV2 {
    type Message = AppMessage;

    fn run(&mut self, sender: mpsc::Sender<Self::Message>) {
        let mut signal = self.signal_sender.subscribe();

        tokio::spawn(async move {
            let mut stream = EventStream::new();
            loop {
                tokio::select! {
                    event_recevied = stream.next() => {
                        if let Some(Ok(ev)) = event_recevied {
                            match ev {
                                TerminalEvent::Key(key_event) => {
                                    let event = if let KeyEvent {
                                        code: KeyCode::Char('c'),
                                        kind: KeyEventKind::Press,
                                        modifiers: KeyModifiers::CONTROL,
                                        ..
                                    } = key_event
                                    {
                                        AppMessage::Quit
                                    } else {
                                        AppMessage::Input(key_event)
                                    };

                                    if sender.send(event).await.is_err() {
                                        break;
                                    }
                                }

                                _ => {}
                            }
                        }
                    }
                    _ = signal.recv() => {
                        // println!("stream dropped!");
                        // drop(stream);
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
