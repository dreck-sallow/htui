use crossterm::event::EventStream;
use futures::StreamExt;
use tokio::task::JoinHandle;

use super::EventSource;

pub struct TerminalSource {
    jh: Option<JoinHandle<()>>,
}

impl TerminalSource {
    pub fn new() -> Self {
        Self { jh: None }
    }
}

impl EventSource for TerminalSource {
    fn start(&mut self, sender: tokio::sync::mpsc::Sender<super::EventMsg>) {
        let jh = tokio::spawn(async move {
            let mut stream = EventStream::new();

            while let Some(Ok(ev)) = stream.next().await {
                match ev {
                    // crossterm::event::Event::FocusGained => todo!(),
                    // crossterm::event::Event::FocusLost => todo!(),
                    // crossterm::event::Event::Mouse(mouse_event) => todo!(),
                    // crossterm::event::Event::Paste(_) => todo!(),
                    // crossterm::event::Event::Resize(_, _) => todo!(),
                    crossterm::event::Event::Key(key_event) => {
                        if sender.send(super::EventMsg::Key(key_event)).await.is_err() {
                            break;
                        }
                    }
                    _ => {}
                }
            }
        });

        self.jh = Some(jh);
    }

    fn finish(&mut self) {
        if let Some(jh) = self.jh.take() {
            jh.abort();
        }
    }
}
