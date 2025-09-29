use std::{fs, os::unix::fs::MetadataExt, path::PathBuf};

use crossterm::event::KeyCode;
use ratatui::{
    layout::Rect,
    style::Stylize,
    widgets::{Block, Clear},
};

use crate::{
    app_project::models::BodyContent,
    programs::tui::{
        common::{input::mode_input::ModeInput, Interactive, UiComposedElement, UiElementV2},
        config::{keybinding::GlobalKeyAction, Config},
        elements::utils::center_area,
    },
};

pub struct BodyBinaryEditor {
    render_area: Rect,
    file: Option<FileDetails>,
    input: ModeInput,
    show_input: bool,
}

impl BodyBinaryEditor {
    pub fn new() -> Self {
        Self {
            render_area: Rect::default(),
            file: None,
            input: ModeInput::new(""),
            show_input: false,
        }
    }

    pub fn body_model(&self) -> BodyContent {
        let txt = match self.file {
            Some(ref details) => details.path_str.as_str(),
            None => "",
        };

        // TODO: handle the case on empty file
        BodyContent::File(PathBuf::from(txt))
    }

    pub fn set_data(&mut self, path: PathBuf) {
        self.file = Self::file_details(path);
        self.input.clear();
        self.show_input = false;
    }

    pub fn render_area(&self) -> Rect {
        self.render_area
    }
}

impl BodyBinaryEditor {
    fn file_details(path: PathBuf) -> Option<FileDetails> {
        if path.try_exists().is_ok() && path.is_file() {
            let path_str = path.to_str().unwrap().to_string();
            Some(FileDetails {
                path_str,
                name: path.file_name().unwrap().to_str().unwrap().to_string(),
                size: fs::metadata(path).unwrap().size(),
            })
        } else {
            None
        }
    }
}

impl<'params> UiComposedElement<'params> for BodyBinaryEditor {
    type Params = &'params Config;

    fn set_area(&mut self, area: Rect, _viewport_area: Rect) {
        self.render_area = area;
        self.input.set_visual_width(
            center_area(
                area,
                ratatui::layout::Constraint::Length(3),
                ratatui::layout::Constraint::Percentage(50),
            )
            .width
                - 2,
        );
    }

    fn draw(&self, _params: Self::Params, frame: &mut ratatui::Frame) {
        match self.file {
            Some(ref details) => {
                let area = center_area(
                    self.render_area,
                    ratatui::layout::Constraint::Length(3),
                    ratatui::layout::Constraint::Percentage(50),
                );

                let mut top = area.top();
                frame.render_widget(
                    format!("PATH: {}", details.path_str).italic(),
                    Rect {
                        height: 1,
                        y: top,
                        ..area
                    },
                );

                top += 1;
                frame.render_widget(
                    format!("NAME: {}", details.name).italic(),
                    Rect {
                        height: 1,
                        y: top,
                        ..area
                    },
                );

                top += 1;
                frame.render_widget(
                    format!("SIZE: {} bytes", details.size).italic(),
                    Rect {
                        height: 1,
                        y: top,
                        ..area
                    },
                );
            }
            None => {
                let txt = "Choose a file pressing 'e' key";
                let area = center_area(
                    self.render_area,
                    ratatui::layout::Constraint::Length(1),
                    ratatui::layout::Constraint::Length(txt.chars().count() as u16),
                );
                frame.render_widget(txt, area);
            }
        }
    }

    fn draw_overlay(&self, config: Self::Params, frame: &mut ratatui::Frame) {
        if self.show_input {
            let area = center_area(
                self.render_area,
                ratatui::layout::Constraint::Length(3),
                ratatui::layout::Constraint::Percentage(50),
            );

            let block = Block::bordered()
                .title("Enter a file path")
                .border_type(ratatui::widgets::BorderType::Thick)
                .border_style(config.theme.border_focus);

            let inner_area = block.inner(area);
            frame.render_widget(Clear, area);
            frame.render_widget(block, area);
            self.input.draw(inner_area, frame);
        }
    }
}

impl<'params> Interactive<'params> for BodyBinaryEditor {
    type Effect = ();

    type Params = &'params Config;

    fn handle_key(
        &mut self,
        config: Self::Params,
        key: crossterm::event::KeyEvent,
    ) -> Self::Effect {
        if self.show_input {
            if let Some(action) = config.keymap.match_global_action(key) {
                match action {
                    GlobalKeyAction::ClosePopup => {
                        self.show_input = false;
                    }
                    GlobalKeyAction::SubmitPopup => {
                        self.file = Self::file_details(PathBuf::from(self.input.txt()));
                        self.show_input = self.file.is_none();
                    }
                    _ => {
                        self.input.handle_key(key);
                    }
                }
            } else {
                self.input.handle_key(key);
            }
        } else {
            if key.code == KeyCode::Char('e') {
                self.show_input = true;
                if let Some(ref details) = self.file {
                    self.input.replace(&details.path_str);
                } else {
                    self.input.replace("");
                }
            }
        }
    }
}

pub struct FileDetails {
    path_str: String,
    name: String,
    size: u64,
}
