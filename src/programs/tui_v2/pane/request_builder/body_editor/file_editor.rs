use std::{os::unix::fs::MetadataExt, path::PathBuf};

use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    text::Span,
    Frame,
};

use crate::programs::tui_v2::{
    common::{elements::utils::center_area, overlays::file_input::FileInput},
    events::DrawSignal,
};

pub struct FileEditor {
    state: Inner,
    file_input: FileInput,
}

impl FileEditor {
    pub fn new(draw_signal: DrawSignal) -> Self {
        Self {
            state: Inner::None,
            file_input: FileInput::new(draw_signal.clone()),
        }
    }

    pub fn is_editing(&self) -> bool {
        self.file_input.is_editing()
    }
}

impl FileEditor {
    pub fn draw(&self, area: Rect, frame: &mut Frame) {
        match &self.state {
            Inner::None => {
                let text = Span::raw("Choose a file");
                let area = center_area(
                    area,
                    Constraint::Length(1),
                    Constraint::Length(text.width() as u16),
                );

                frame.render_widget(text, area);
            }
            Inner::File { path, details } => {
                let (title_area, content_area) = {
                    let center_area =
                        center_area(area, Constraint::Length(5), Constraint::Percentage(80));

                    (
                        Rect {
                            height: 1,
                            ..center_area
                        },
                        Rect {
                            y: center_area.y + 2,
                            height: center_area.height - 2,
                            ..center_area
                        },
                    )
                };

                let title = Span::raw("File details").blue();
                let title_w = title.width() as u16;
                frame.render_widget(
                    title,
                    center_area(
                        title_area,
                        Constraint::Length(1),
                        Constraint::Percentage(title_w),
                    ),
                );

                let values = [
                    Span::raw(&details.name),
                    Span::raw(&details.size),
                    Span::raw(&details.path),
                ];

                // Draw content
                let [mut keys_area, mut values_area] =
                    Layout::horizontal([Constraint::Length(10), Constraint::Percentage(60)]).areas(
                        center_area(
                            content_area,
                            Constraint::Length(3),
                            Constraint::Length(
                                6 + values
                                    .iter()
                                    .fold(2, |acc, sp| (sp.width() as u16).max(acc)),
                            ),
                        ),
                    );

                for sp in ["Name", "Size", "Path"] {
                    frame.render_widget(Span::raw(sp).blue(), keys_area);
                    keys_area.y += 1;
                }

                for sp in [&details.name, &details.size, &details.path] {
                    frame.render_widget(Span::raw(sp), values_area);
                    values_area.y += 1;
                }
            }
            Inner::Error => {
                let text = Span::raw("Failed to read the file");
                let area = center_area(
                    area,
                    Constraint::Length(1),
                    Constraint::Length(text.width() as u16),
                );

                frame.render_widget(text, area);
            }
        }
    }

    pub fn draw_overlay(&self, frame: &mut Frame) {
        if self.file_input.is_visible() {
            self.file_input.draw(frame);
        }
    }

    pub async fn handle_key(&mut self, key: KeyEvent) {
        if self.file_input.is_visible() {
            match key.code {
                crossterm::event::KeyCode::Enter => {
                    self.file_input.handle_key(key).await;

                    let Some(path) = self.file_input.path() else {
                        return;
                    };

                    if path.is_file() {
                        match tokio::fs::metadata(&path).await {
                            Ok(m) => {
                                let name = path.file_name().unwrap().to_str().unwrap().to_string();
                                let path_str = path.to_str().unwrap().to_string();

                                self.state = Inner::File {
                                    path,
                                    details: FileDetails {
                                        name,
                                        size: m.size().to_string(),
                                        path: path_str,
                                    },
                                };
                            }
                            Err(_) => self.state = Inner::Error,
                        }
                        self.file_input.hide();
                    }
                }
                crossterm::event::KeyCode::Esc if !self.file_input.is_editing() => {
                    self.file_input.hide();
                }
                _ => {
                    self.file_input.handle_key(key).await;
                }
            }
        } else {
            match key.code {
                crossterm::event::KeyCode::Char('f') => {
                    self.file_input.show(PathBuf::new());
                }
                _ => {}
            }
        }
    }
}

pub enum Inner {
    None,
    File { path: PathBuf, details: FileDetails },
    Error,
}

pub struct FileDetails {
    name: String,
    size: String,
    path: String,
}
