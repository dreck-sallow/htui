use std::{cell::RefCell, fs, path::PathBuf, rc::Rc};

use arboard::Clipboard;
use file_input::FileInput;
use hexdump::HexDump;
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    widgets::Widget,
};

use crate::programs::tui::{
    common::{component::Interactive, InteractiveElement, UiElement},
    config::{keybinding, Config},
};

mod file_input;
mod hexdump;

pub struct BinaryViewer {
    hexdump: HexDump,
    file_path: Option<PathBuf>,
    file_input: FileInput,
    _view_area: Rect,
}

impl BinaryViewer {
    pub fn new(bytes: &[u8], path: Option<PathBuf>, view_area: Rect) -> Self {
        let mut this = Self {
            hexdump: HexDump::new(bytes, Style::default().italic().bold()),
            file_input: FileInput::new(path.as_ref().map_or("", |p| p.to_str().unwrap())),
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
                let path_txt = path.to_str().unwrap();
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
impl<'a> InteractiveElement<'a> for BinaryViewer {
    type Params = (Rc<Config>, Rc<RefCell<Clipboard>>, SliceBytes<'a>);

    fn handle_key(
        &mut self,
        (config, clipboard, bytes): Self::Params,
        key: crossterm::event::KeyEvent,
    ) {
        if self.file_input.is_show() {
            if let Some(effect) = self.file_input.on_key(key, config) {
                match effect {
                    file_input::FileInputEffect::NewPath(path_buf) => {
                        match &self.file_path {
                            Some(path) => {
                                let _ = fs::rename(path, &path_buf);
                            }
                            None => {
                                let _ = fs::write(&path_buf, bytes);
                            }
                        }

                        self.file_path = Some(path_buf);
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
    }
}

// pub struct

// impl Interactive for BinaryViewer {
//     type Effect = ();

//     type Params = (Rc<Config>, Rc<RefCell<Clipboard>>);

//     fn on_key(
//         &mut self,
//         key: crossterm::event::KeyEvent,
//         (config, clipboard): Self::Params,
//     ) -> Option<Self::Effect> {
//         if self.file_input.is_show() {
//             if let Some(effect) = self.file_input.on_key(key, config) {
//                 match effect {
//                     file_input::FileInputEffect::NewPath(path_buf) => {
//                         self.file_path = Some(path_buf);
//                     }
//                 }
//             }
//         } else {
//             let consumed = config
//                 .keymap
//                 .match_global_action(key)
//                 .map_or(false, |action| match action {
//                     keybinding::GlobalKeyAction::MoveDown => {
//                         self.hexdump.next();
//                         true
//                     }
//                     keybinding::GlobalKeyAction::MoveUp => {
//                         self.hexdump.previous();
//                         true
//                     }
//                     keybinding::GlobalKeyAction::CopyToClipboard => {
//                         if let Some(txt) = self.hexdump.line_to_txt() {
//                             let _ = clipboard.borrow_mut().set_text(txt);
//                         }

//                         true
//                     }
//                     _ => false,
//                 });

//             if !consumed {
//                 if let Some(action) = config.keymap.match_response_viewer_action(key) {
//                     match action {
//                         // keybinding::ResponseViewerAction::SaveBytes => {
//                         //     match self.file_path {
//                         //         Some(ref path) => {
//                         //             let _ = fs::write(path, []);
//                         //         }
//                         //         None => {
//                         //             self.file_input.hidden();
//                         //         }
//                         //     }
//                         //     // if self.file_path.is_none() {
//                         //     //     self.show_input = true;
//                         //     // } else {
//                         //     //     fs::write(, contents)
//                         //     //     // TODO: write to path using the file path
//                         //     // }
//                         // }
//                         keybinding::ResponseViewerAction::EditFilePath => {
//                             self.file_input.show();
//                         }
//                     }
//                 }
//             }
//         }

//         None
//     }
// }
