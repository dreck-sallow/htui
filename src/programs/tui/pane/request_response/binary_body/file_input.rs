use std::{path::PathBuf, rc::Rc};

use ratatui::{
    layout::{Constraint, Margin, Rect},
    style::Style,
    widgets::{Block, Clear},
};

use crate::programs::tui::{
    common::{component::Interactive, input::mode_input::ModeInput, UiElement},
    config::{keybinding, Config},
    elements::utils::center_area,
};

pub struct FileInput {
    show_input: bool,
    input: ModeInput,
}

impl FileInput {
    pub fn new(path: &str) -> Self {
        Self {
            show_input: false,
            input: ModeInput::new(path),
        }
    }

    pub fn is_show(&self) -> bool {
        self.show_input
    }

    pub fn show(&mut self) {
        self.show_input = true;
    }

    pub fn hidden(&mut self) {
        self.show_input = false;
    }
}

impl UiElement for FileInput {
    type Params = (Rc<Config>, Rect);

    fn set_area(&mut self, area: ratatui::prelude::Rect) {
        self.input.set_area(
            center_area(area, Constraint::Length(3), Constraint::Percentage(50))
                .inner(Margin::new(1, 1)),
        );
        // self.select_mode_input.set_area(
        //     center_area(area, Constraint::Length(3), Constraint::Percentage(50))
        //         .inner(Margin::new(1, 1)),
        // );
    }

    fn draw(&self, _params: Self::Params, _frame: &mut ratatui::Frame) {}

    fn draw_overlay(&self, (config, area): Self::Params, frame: &mut ratatui::Frame) {
        if self.show_input {
            let center_area = center_area(area, Constraint::Length(3), Constraint::Percentage(50));
            frame.render_widget(Clear, center_area);

            let block = Block::bordered()
                .title("| File path |")
                .border_type(ratatui::widgets::BorderType::Thick)
                .border_style(Style::default().fg(config.theme.border_focus));

            frame.render_widget(block, center_area);
            self.input.draw((), frame);
        }
    }
}

impl Interactive for FileInput {
    type Effect = FileInputEffect;

    type Params = Rc<Config>;

    fn on_key(
        &mut self,
        key: crossterm::event::KeyEvent,
        config: Self::Params,
    ) -> Option<Self::Effect> {
        let consumed = match config.keymap.match_global_action(key) {
            Some(action) => match action {
                keybinding::GlobalKeyAction::SubmitPopup => {
                    let txt = self.input.txt();
                    self.show_input = false;
                    return Some(FileInputEffect::NewPath(PathBuf::from(txt)));
                }
                keybinding::GlobalKeyAction::ClosePopup if !self.input.mode().is_write_mode() => {
                    self.show_input = false;
                    if self.input.txt().is_empty() {
                        self.input.clear();
                    }
                    true
                }
                _ => false,
            },
            None => false,
        };

        if !consumed {
            self.input.handle_key(key);
        }

        // if !consumed {
        //     self.input.handle_key(key);
        //     let txt = self.input.txt().to_string();
        //     let path = Path::new(&txt);

        //     if let (Some(cache_parent), Some(parent)) = (self.last_path.parent(), path.parent()) {
        //         if cache_parent != parent {
        //             self.last_path = path.to_path_buf();

        //             match list_path_options(path) {
        //                 Ok(list) => self.select_mode_input.set_options(list),
        //                 Err(_e) => {}
        //             }
        //         }
        //     } else {
        //         self.last_path = path.to_path_buf();
        //     }

        //     if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        //         let name_len = name.chars().count();
        //         let input_len = txt.chars().count();
        //         self.select_mode_input
        //             .set_range_replace((input_len.saturating_sub(name_len), input_len));
        //     }
        // } else {
        //     self.select_mode_input.set_options(Vec::new());
        // }

        None
    }
}

pub enum FileInputEffect {
    NewPath(PathBuf),
}

// fn list_path_options(path: &Path) -> std::io::Result<Vec<SelectOption>> {
//     let mut options = Vec::new();

//     if let Some(parent_path) = path.parent() {
//         for entry in fs::read_dir(parent_path)? {
//             let entry = entry?;
//             let name = entry.file_name().into_string().unwrap();
//             options.push(SelectOption::new(name.clone(), name));
//         }
//     }

//     Ok(options)
// }
