use std::{
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
    time::Duration,
};

use crossterm::event::KeyEvent;
use ratatui::{
    layout::Rect,
    text::Span,
    widgets::{Block, Borders},
    Frame,
};
use tokio::{sync::mpsc::Sender, task::JoinHandle, time::Instant};

use crate::programs::tui_v2::{common::list, events::DrawSignal};

use super::popup_input::PopupInput;

pub type EntryPaths = Arc<RwLock<Vec<String>>>;

pub struct FileInput {
    popup_input: PopupInput,

    task: JoinHandle<()>,
    paths: EntryPaths,
    tx: Sender<PathBuf>,

    paths_idx: Option<usize>,
}

impl FileInput {
    pub fn new(draw_signal: DrawSignal) -> Self {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<PathBuf>(10);

        let paths = Arc::new(RwLock::new(Vec::new()));

        let clone_paths = Arc::clone(&paths);
        let task = tokio::spawn(async move {
            loop {
                let Some(path) = debounce_receiver(&mut rx, Duration::from_millis(200)).await
                else {
                    return;
                };

                // the current path is new!;
                *clone_paths.write().unwrap() = Self::search_list(path).await;
                draw_signal.draw();
            }
        });

        Self {
            popup_input: PopupInput::new(),
            paths_idx: None,
            task,
            paths,
            tx,
        }
    }

    async fn search_list<P: AsRef<Path>>(path: P) -> Vec<String> {
        let path = path.as_ref();

        let dir_path = if path.is_dir() {
            Some(path)
        } else {
            path.parent()
        };

        let Some(read_path) = dir_path else {
            return Vec::new();
        };

        let Ok(mut read_dir) = tokio::fs::read_dir(read_path).await else {
            return vec![];
        };

        let mut list = Vec::new();

        while let Ok(Some(entry)) = read_dir.next_entry().await {
            if let Ok(p) = entry.path().into_os_string().into_string() {
                list.push(p);
            }
        }

        list
    }

    pub fn is_visible(&self) -> bool {
        self.popup_input.is_visible()
    }

    pub fn show(&mut self, path: PathBuf) {
        self.popup_input.show(path.to_str().unwrap());
        let _ = self.tx.send(path);
    }

    pub fn hide(&mut self) {
        self.popup_input.hide();
    }
}

impl FileInput {
    pub fn draw(&self, frame: &mut Frame) {
        let render_area = self.popup_input.draw_center(
            "Choose File",
            super::popup_input::CenterArea {
                height: ratatui::layout::Constraint::Length(3),
                width: ratatui::layout::Constraint::Percentage(40),
            },
            frame,
        );

        let Ok(items) = self.paths.try_read() else {
            return;
        };

        if items.is_empty() {
            return;
        }

        let list_block_area = Rect {
            y: render_area.bottom(),
            height: 5,
            ..render_area
        };

        let block = Block::new().borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM);

        let list_area = block.inner(list_block_area);

        frame.render_widget(block, list_block_area);

        let mut line = Rect {
            x: list_area.left(),
            y: list_area.top(),
            width: list_area.width,
            height: 1,
        };

        // Render list
        for (i, _y) in (list_area.top()..list_area.bottom()).enumerate() {
            let Some(item) = items.get(i) else { break };
            frame.render_widget(Span::raw(item), line);
            line.y += 1;
        }
    }

    pub async fn handle_key(&mut self, key: KeyEvent) {
        if !self.popup_input.is_visible() {
            return;
        }

        if self.popup_input.is_editing() {
            self.popup_input.handle_input_key(key);
            let _ = self.tx.send(PathBuf::from(self.popup_input.value())).await;
        } else {
            match key.code {
                crossterm::event::KeyCode::Tab | crossterm::event::KeyCode::Down => {
                    self.paths_idx = list::next(
                        self.paths_idx,
                        self.paths.try_read().map(|itms| itms.len()).unwrap_or(0),
                    )
                }
                crossterm::event::KeyCode::BackTab | crossterm::event::KeyCode::Up => {
                    self.paths_idx = list::prev(self.paths_idx);
                }
                crossterm::event::KeyCode::Enter => {}
                _ => {
                    self.popup_input.handle_input_key(key);
                }
            }
        }
    }
}

async fn debounce_receiver<T>(
    rx: &mut tokio::sync::mpsc::Receiver<T>,
    delay: Duration,
) -> Option<T> {
    let mut val = rx.recv().await?;

    while let Ok(opt) = tokio::time::timeout_at(Instant::now() + delay, rx.recv()).await {
        val = opt?;
    }

    Some(val)
}
