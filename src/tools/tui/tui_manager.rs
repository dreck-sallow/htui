use std::io::{stdout, Result, Stdout};

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use futures_util::{stream::StreamExt, FutureExt};

use ratatui::{
    crossterm::event::{Event, KeyEvent},
    prelude::CrosstermBackend,
    Terminal,
};
use tokio::{
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

pub struct TuiManager {
    pub terminal: Terminal<CrosstermBackend<Stdout>>,
    pub cancellation_token: CancellationToken,
    pub tick_rate: f64,
    pub frame_rate: f64,
    pub event_rx: UnboundedReceiver<TuiEvent>,
    pub event_tx: UnboundedSender<TuiEvent>,
    pub task: JoinHandle<()>,
}

impl TuiManager {
    pub fn new(tick_rate: f64, frame_rate: f64) -> Result<Self> {
        let terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
        let cancellation_token = CancellationToken::new();

        let (tx, rx) = mpsc::unbounded_channel::<TuiEvent>();

        let task = tokio::spawn(async {});

        Ok(Self {
            terminal,
            cancellation_token,
            tick_rate,
            frame_rate,
            event_rx: rx,
            event_tx: tx,
            task,
        })
    }

    pub fn start(&mut self) {
        let tick_delay = std::time::Duration::from_secs_f64(1.0 / self.tick_rate);
        let render_delay = std::time::Duration::from_secs_f64(1.0 / self.frame_rate);

        let cancellation_token_cloned = self.cancellation_token.clone();

        let sender_clone = self.event_tx.clone();

        self.task = tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();

            let mut tick_interval = tokio::time::interval(tick_delay);
            let mut render_interval = tokio::time::interval(render_delay);

            loop {
                let tick_delay = tick_interval.tick();
                let render_delay = render_interval.tick();

                let crossterm_event = reader.next().fuse();

                //TODO: send the TuiEvent::Init; // :)

                tokio::select! {
                    _ = cancellation_token_cloned.cancelled() => {
                        break;
                    },
                    event = crossterm_event => {
                        if let Some(Ok(ev)) = event {
                            match ev {
                                Event::Key(key) => {
                                    sender_clone.send(TuiEvent::Key(key)).unwrap();
                                },
                                _ => {}
                            }
                        }
                    },
                    _ = tick_delay => {
                        sender_clone.send(TuiEvent::Tick).unwrap();
                    },
                    _ = render_delay => {
                        sender_clone.send(TuiEvent::Render).unwrap();
                    }

                }
            }
        });
    }

    pub fn enter(&mut self) -> Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen)?;
        self.start();
        Ok(())
    }

    pub fn stop(&mut self) {
        self.cancellation_token.cancel();
    }

    pub fn exit(&mut self) -> Result<()> {
        self.stop();
        if crossterm::terminal::is_raw_mode_enabled()? {
            execute!(stdout(), LeaveAlternateScreen)?;
            crossterm::terminal::disable_raw_mode()?;
        }

        Ok(())
    }

    pub async fn next(&mut self) -> Option<TuiEvent> {
        self.event_rx.recv().await
    }
}

#[derive(Debug)]
pub enum TuiEvent {
    Error,
    Tick,
    Render,
    Key(KeyEvent),
}
