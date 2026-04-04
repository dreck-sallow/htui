use std::{
    ops::Not,
    path::{Path, PathBuf, MAIN_SEPARATOR},
    sync::{Arc, RwLock},
    time::Duration,
};

use crossterm::event::KeyEvent;
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Borders},
    Frame,
};
use tokio::{sync::mpsc::Sender, task::JoinHandle, time::Instant};

use crate::programs::tui_v2::{
    common::{elements::utils::center_area, list, ui_elements::list::UiList},
    events::DrawSignal,
};

use super::popup_input::PopupInput;

type EntryPaths = Arc<RwLock<PathSelector>>;

pub struct FileInput {
    popup_input: PopupInput,

    task: JoinHandle<()>,
    paths: EntryPaths,
    tx: Sender<PathBuf>,
}

impl FileInput {
    pub fn new(draw_signal: DrawSignal) -> Self {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<PathBuf>(10);

        let paths = Arc::new(RwLock::new(PathSelector::new()));

        let clone_paths = Arc::clone(&paths);
        let task = tokio::spawn(async move {
            loop {
                let Some(path) = debounce_receiver(&mut rx, Duration::from_millis(200)).await
                else {
                    return;
                };

                // the current path is new!;
                let list = Self::search_list(path).await;
                clone_paths.write().unwrap().set_list(list);

                draw_signal.draw();
            }
        });

        Self {
            popup_input: PopupInput::new(),
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

    pub fn is_editing(&self) -> bool {
        self.popup_input.is_editing()
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
        let (input_area, list_block_area) = {
            let area = center_area(
                frame.area(),
                ratatui::layout::Constraint::Length(12),
                ratatui::layout::Constraint::Percentage(40),
            );

            (
                Rect { height: 3, ..area },
                Rect {
                    y: area.top() + 3,
                    height: 9,
                    ..area
                },
            )
        };

        // let render_area = self.popup_input.draw_center(
        self.popup_input.draw("Choose File", input_area, frame);

        let Ok(paths) = self.paths.try_read() else {
            return;
        };

        let (idx, items) = paths.selection();
        drop(paths);

        if items.is_empty() {
            return;
        }

        // let list_block_area = Rect {
        //     y: render_area.bottom(),
        //     height: 5,
        //     ..render_area
        // };

        let block = Block::new().borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM);

        let list_area = block.inner(list_block_area);
        let items = {
            let mut itms = Vec::with_capacity(items.capacity());
            for itm in items {
                itms.push(Line::raw(itm));
            }
            itms
        };

        frame.render_widget(block, list_block_area);

        // TODO: map all items for show only a page?
        let list = UiList::new()
            .with_idx(idx)
            .with_items(items.into())
            .with_highlight_style(Style::default().on_light_blue());

        list.draw(list_area, frame);
    }

    async fn handle_input_key(&mut self, key: KeyEvent) {
        let previous_path = PathBuf::from(self.popup_input.value());
        let Some(act) = self.popup_input.handle_input_key(key) else {
            return;
        };

        if act.is_mutation() {
            let path = PathBuf::from(self.popup_input.value());

            if previous_path == path {
                return;
            }

            if self.popup_input.value().is_empty() {
                self.paths.write().unwrap().set_list(Vec::new());
            } else if self.popup_input.value().chars().last().unwrap() == MAIN_SEPARATOR {
                let _ = self.tx.send(path).await;
            } else {
                if let Some(last) = path
                    .components()
                    .last()
                    .and_then(|cmp| cmp.as_os_str().to_str())
                {
                    self.paths.write().unwrap().filter_by_name(last);
                }
            }
        }
    }

    pub async fn handle_key(&mut self, key: KeyEvent) {
        if !self.popup_input.is_visible() {
            return;
        }

        if self.popup_input.is_editing() {
            self.handle_input_key(key).await;
        } else {
            match key.code {
                crossterm::event::KeyCode::Tab | crossterm::event::KeyCode::Down => {
                    self.paths.write().unwrap().next();
                }
                crossterm::event::KeyCode::BackTab | crossterm::event::KeyCode::Up => {
                    self.paths.write().unwrap().prev();
                }
                crossterm::event::KeyCode::Enter => {}
                _ => {
                    self.handle_input_key(key).await;
                }
            }
        }
    }
}

struct PathSelector {
    selected: Option<usize>,
    list: Vec<String>,
    options: Vec<usize>,
}

impl PathSelector {
    pub fn new() -> Self {
        Self {
            selected: None,
            list: Vec::new(),
            options: Vec::new(),
        }
    }

    pub fn set_list(&mut self, list: Vec<String>) {
        self.options = list.iter().enumerate().map(|(i, _)| i).collect();
        self.list = list;
        self.selected = self.options.is_empty().not().then_some(0);
    }

    pub fn filter_by_name(&mut self, name: &str) {
        self.options = Vec::new();

        for (i, itm) in self.list.iter().enumerate() {
            if itm.starts_with(name) {
                self.options.push(i);
            }
        }
    }

    pub fn next(&mut self) {
        self.selected = list::next(self.selected, self.options.len());
    }

    pub fn prev(&mut self) {
        self.selected = list::prev(self.selected);
    }

    pub fn parts(&self) -> (Option<usize>, Vec<String>) {
        (self.selected.clone(), self.list.clone())
    }

    pub fn selection(&self) -> (Option<usize>, Vec<String>) {
        (
            self.selected.clone(),
            self.options.iter().map(|i| self.list[*i].clone()).collect(),
        )
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
