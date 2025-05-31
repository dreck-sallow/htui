use ratatui::{
    layout::{Constraint, Rect},
    style::Style,
    widgets::{Block, Clear},
};

use crate::programs::tui::{
    common::{input::mode_input::ModeInput, Interactive, UiComposedElement, UiElementV2},
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
}

impl<'params> UiComposedElement<'params> for FileInput {
    type Params = (&'params Config, Rect);

    fn set_area(&mut self, area: Rect, _viewport_area: Rect) {
        self.input.set_visual_width(area.width.saturating_sub(2));
    }

    fn draw(&self, _params: Self::Params, _frame: &mut ratatui::Frame) {
        unimplemented!()
    }

    fn draw_overlay(&self, (config, area): Self::Params, frame: &mut ratatui::Frame) {
        if self.show_input {
            let center_area = center_area(area, Constraint::Length(3), Constraint::Percentage(50));
            frame.render_widget(Clear, center_area);

            let block = Block::bordered()
                .title("| File path |")
                .border_type(ratatui::widgets::BorderType::Thick)
                .border_style(Style::default().fg(config.theme.border_focus));

            let inner_width = block.inner(center_area);

            frame.render_widget(block, center_area);
            self.input.draw(inner_width, frame);
        }
    }
}

impl<'params> Interactive<'params> for FileInput {
    type Effect = Option<FileInputEffect>;

    type Params = &'params Config;

    fn handle_key(
        &mut self,
        config: Self::Params,
        key: crossterm::event::KeyEvent,
    ) -> Self::Effect {
        let consumed = match config.keymap.match_global_action(key) {
            Some(action) => match action {
                keybinding::GlobalKeyAction::SubmitPopup => {
                    let txt = self.input.txt();
                    self.show_input = false;
                    return Some(FileInputEffect::NewPath(txt.to_string()));
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
        None
    }
}

pub enum FileInputEffect {
    NewPath(String),
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
