use std::{cell::RefCell, fs, rc::Rc};

use arboard::Clipboard;
use file_input::FileInput;
use hexdump::HexDump;
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    widgets::Widget,
};

use crate::{
    app_project::models::ResponseFilePath,
    programs::tui::{
        common::{component::Interactive, InteractiveElementEff, UiElement},
        config::{keybinding, Config},
    },
};

mod file_input;
mod hexdump;

pub struct BinaryViewer {
    hexdump: HexDump,
    file_path: Option<String>,
    file_input: FileInput,
    _view_area: Rect,
}

impl BinaryViewer {
    pub fn new(bytes: &[u8], file_path: &ResponseFilePath, view_area: Rect) -> Self {
        let (path, path_str) = match file_path {
            ResponseFilePath::Temp(temp_path) => {
                let txt = temp_path.to_str().map(|f| f.to_string()).unwrap();
                (Some(txt.clone()), txt)
            }
            ResponseFilePath::Saved(path_buf) => {
                let txt = path_buf.to_str().map(|f| f.to_string()).unwrap();
                (Some(txt.to_string()), txt)
            }
            ResponseFilePath::Null => (None, "".to_string()),
        };

        let mut this = Self {
            hexdump: HexDump::new(bytes, Style::default().italic().bold()),
            file_input: FileInput::new(&path_str),
            file_path: path,
            _view_area: view_area,
        };

        this.set_area(view_area);
        this
    }
}

impl UiElement for BinaryViewer {
    type Params = Rc<Config>;

    fn set_area(&mut self, area: Rect) {
        self._view_area = area;
        self.file_input.set_area(area);
    }

    fn draw(&self, _params: Self::Params, frame: &mut ratatui::Frame) {
        let mut area = self._view_area;
        let buf = frame.buffer_mut();
        let left = buf
            .set_stringn(
                area.left() + 1,
                area.top(),
                "Save as: ",
                area.width as usize,
                Style::default(),
            )
            .0;

        match self.file_path {
            Some(ref path) => {
                let path_txt = path.as_str();
                buf.set_stringn(
                    left,
                    area.top(),
                    path_txt,
                    path_txt.chars().count(),
                    Style::default().green().underlined(),
                );
            }
            None => {
                buf.set_stringn(
                    left,
                    area.top(),
                    "Not path specified",
                    17,
                    Style::default().italic().dark_gray().underlined(),
                );
            }
        }

        area.y += 2;
        (&self.hexdump).render(Rect { y: area.y, ..area }, buf);
    }

    fn draw_overlay(&self, config: Self::Params, frame: &mut ratatui::Frame) {
        if self.file_input.is_show() {
            self.file_input
                .draw_overlay((config, self._view_area), frame);
        }
    }
}

type SliceBytes<'a> = &'a [u8];
impl<'a> InteractiveElementEff<'a> for BinaryViewer {
    type Effect = BinaryViewerEffect;
    type Params = (Rc<Config>, Rc<RefCell<Clipboard>>, SliceBytes<'a>);

    fn handle_key(
        &mut self,
        (config, clipboard, bytes): Self::Params,
        key: crossterm::event::KeyEvent,
    ) -> Self::Effect {
        if self.file_input.is_show() {
            if let Some(effect) = self.file_input.on_key(key, config) {
                match effect {
                    file_input::FileInputEffect::NewPath(new_path) => {
                        match &self.file_path {
                            Some(path) => {
                                let _ = fs::rename(path, &new_path);
                            }
                            None => {
                                let _ = fs::write(&new_path, bytes);
                            }
                        }

                        self.file_path = Some(new_path.clone());
                        return BinaryViewerEffect::NewFilePath(new_path);
                    }
                }
            }
        } else {
            let consumed = config
                .keymap
                .match_global_action(key)
                .map_or(false, |action| match action {
                    keybinding::GlobalKeyAction::MoveDown => {
                        self.hexdump.next();
                        true
                    }
                    keybinding::GlobalKeyAction::MoveUp => {
                        self.hexdump.previous();
                        true
                    }
                    keybinding::GlobalKeyAction::CopyToClipboard => {
                        if let Some(txt) = self.hexdump.line_to_txt() {
                            let _ = clipboard.borrow_mut().set_text(txt);
                        }

                        true
                    }
                    _ => false,
                });

            if !consumed {
                if let Some(action) = config.keymap.match_response_viewer_action(key) {
                    match action {
                        keybinding::ResponseViewerAction::EditFilePath => {
                            self.file_input.show();
                        }
                    }
                }
            }
        }
        BinaryViewerEffect::Noop
    }
}

pub enum BinaryViewerEffect {
    NewFilePath(String),
    Noop,
}
