use crossterm::event::KeyEvent;
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::Span,
    Frame,
};

use crate::programs::tui_v2::{common::table_grid::UiTableGrid, pane::state::BodyForm};

pub struct FormDataEditor {
    state: BodyForm,
}

impl Default for FormDataEditor {
    fn default() -> Self {
        Self::new(BodyForm::default())
    }
}

impl FormDataEditor {
    pub fn new(state: BodyForm) -> Self {
        Self { state }
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

    pub fn handle_key(&mut self, key: KeyEvent) {
        // let table = &mut self.state;
        // match key.code {
        //     crossterm::event::KeyCode::Char(ch) => match ch {
        //         'j' => table.next_row(),
        //         'k' => table.prev_row(),
        //         'h' => table.prev_col(),
        //         'l' => table.next_col(),
        //         'n' => table.add_item(ParamItem::empty()),
        //         'd' => {
        //             table.delete_current();
        //         }
        //         'e' => {
        //             let idx = table.idx.clone();
        //             if let Some(itm) = table.current_mut() {
        //                 match idx.unwrap().1 {
        //                     0 => {
        //                         itm.toggle_enable();
        //                     }
        //                     1 => {
        //                         self.edit_popup.edit_key(&itm.key);
        //                     }
        //                     2 => {
        //                         self.edit_popup.edit_value(&itm.value);
        //                     }
        //                     _ => unreachable!(),
        //                 }
        //             }
        //         }
        //         _ => {}
        //     },
        //     _ => {}
        // }
    }
}
