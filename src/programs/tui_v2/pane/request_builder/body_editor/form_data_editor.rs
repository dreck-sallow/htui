use std::path::PathBuf;

use crossterm::event::KeyEvent;
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::Span,
    Frame,
};

use crate::programs::tui_v2::{
    common::{overlays::file_input::FileInput, table_grid::UiTableGrid},
    events::DrawSignal,
    pane::{
        request_builder::edit_param_table::EditParamTablePopup,
        state::{BodyForm, ParamItem},
    },
};

pub struct FormDataEditor {
    edit_text_field: EditParamTablePopup,
    edit_file: FileInput,
    state: BodyForm,
}

// impl Default for FormDataEditor {
//     fn default() -> Self {
//         Self::new(BodyForm::default())
//     }
// }

impl FormDataEditor {
    pub fn new(state: BodyForm, draw_signal: DrawSignal) -> Self {
        Self {
            state,
            edit_text_field: EditParamTablePopup::new(),
            edit_file: FileInput::new(draw_signal),
        }
    }

    pub fn from_default(draw_signal: DrawSignal) -> Self {
        Self::new(BodyForm::default(), draw_signal)
    }
}

impl FormDataEditor {
    pub fn draw(&self, area: Rect, frame: &mut Frame) {
        UiTableGrid::new(
            ["Enable".blue(), "Header Name".blue(), "Header Value".blue()],
            [0.2, 0.4, 0.4],
        )
        .with_rows(
            self.state
                .items
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    {
                        [
                            Span::raw(if p.enable { "Yes" } else { "No" }),
                            Span::raw(if !p.key.is_empty() { &p.key } else { "-" }),
                            Span::raw(if !p.value.is_empty() { &p.value } else { "-" }).style(
                                if self.state.are_files.contains(&i) {
                                    Style::default().bold().italic().underlined()
                                } else {
                                    Style::default()
                                },
                            ),
                        ]
                    }
                })
                .collect(),
        )
        .with_index(self.state.idx, Style::default().on_dark_gray())
        .draw(area, frame);
    }

    pub fn draw_overlay(&self, frame: &mut Frame) {
        if self.edit_text_field.is_visible() {
            self.edit_text_field.draw(frame);
        } else if self.edit_file.is_visible() {
            self.edit_file.draw(frame);
        }
    }

    pub async fn handle_key(&mut self, key: KeyEvent) {
        if self.edit_text_field.is_visible() {
            match key.code {
                crossterm::event::KeyCode::Enter => {}
                crossterm::event::KeyCode::Esc => {
                    self.edit_text_field.hide();
                }
                _ => {
                    // self.edit_text_field.handle_key(key, &mut self.state)
                }
            }
        } else if self.edit_file.is_visible() {
            match key.code {
                crossterm::event::KeyCode::Enter => {}
                crossterm::event::KeyCode::Esc if !self.edit_file.is_editing() => {
                    self.edit_file.hide();
                }
                _ => {
                    self.edit_file.handle_key(key).await;
                }
            }
        } else {
            let table = &mut self.state;
            match key.code {
                crossterm::event::KeyCode::Char(ch) => match ch {
                    'j' => table.next_row(),
                    'k' => table.prev_row(),
                    'h' => table.prev_col(),
                    'l' => table.next_col(),
                    'n' => table.add_item(ParamItem::empty()),
                    'd' => {
                        table.delete_current();
                    }
                    'e' => {
                        let idx = table.idx.clone();
                        if let Some(itm) = table.current_mut() {
                            match idx.unwrap().1 {
                                0 => {
                                    itm.toggle_enable();
                                }
                                1 => {
                                    self.edit_text_field.edit_key(&itm.key);
                                }
                                2 => {
                                    self.edit_text_field.edit_value(&itm.value);
                                }
                                _ => unreachable!(),
                            }
                        }
                    }
                    'f' => {
                        let idx = table.idx.clone().unwrap();
                        if let Some(itm) = table.current_mut() {
                            match idx.1 {
                                2 => {
                                    self.edit_file.show(PathBuf::from(&itm.value));
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
}
