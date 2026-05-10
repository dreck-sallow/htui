use crossterm::event::KeyEvent;
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::Span,
    Frame,
};

use crate::programs::tui_v2::{
    common::table_grid::UiTableGrid,
    pane::state_v2::{param_item::ParamItem, responses::ReadonlyHeader, table::TableState},
};

use super::edit_param_table_popup::EditParamTablePopup;

pub struct ParamsTableUi {
    edit_popup: EditParamTablePopup,
}

impl ParamsTableUi {
    pub fn new() -> Self {
        Self {
            edit_popup: EditParamTablePopup::new(),
        }
    }

    pub fn is_editing(&self) -> bool {
        self.edit_popup.is_visible()
    }
}

impl ParamsTableUi {
    pub fn draw(&self, table: &TableState<ParamItem>, area: Rect, frame: &mut Frame) {
        params_to_ui(table).draw(area, frame);
    }

    pub fn draw_overlay(&self, frame: &mut Frame) {
        if self.edit_popup.is_visible() {
            self.edit_popup.draw(frame);
        }
    }

    pub fn handle_table_key(&mut self, key: KeyEvent, table: &mut TableState<ParamItem>) {
        if self.edit_popup.is_visible() {
            self.edit_popup.handle_key(key, table);
        } else {
            match key.code {
                crossterm::event::KeyCode::Char(ch) => match ch {
                    'j' => table.next_row(),
                    'k' => table.prev_row(),
                    'h' => table.prev_cell(),
                    'l' => table.next_cell(),
                    'n' => table.add_row(ParamItem::empty()),
                    'd' => {
                        table.remove_current_row();
                    }
                    'e' => {
                        let idx = table.idx();
                        if let Some(itm) = table.current_mut() {
                            match idx.as_cell().unwrap().1 {
                                0 => {
                                    itm.toggle_enable();
                                }
                                1 => {
                                    self.edit_popup.edit_key(&itm.key);
                                }
                                2 => {
                                    self.edit_popup.edit_value(&itm.value);
                                }
                                _ => unreachable!(),
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

pub fn params_to_ui(params: &TableState<ParamItem>) -> UiTableGrid<'_, '_, 3> {
    UiTableGrid::new(
        ["Enable".blue(), "Header Name".blue(), "Header Value".blue()],
        [0.2, 0.4, 0.4],
    )
    .with_rows(
        params
            .items()
            .iter()
            .map(|p| {
                {
                    [
                        Span::raw(if p.enable { "Yes" } else { "No" }),
                        Span::raw(if p.key.len() > 1 { &p.key } else { "-" }),
                        Span::raw(if p.value.len() > 1 { &p.value } else { "-" }),
                    ]
                }
            })
            .collect(),
    )
    .with_index(params.idx().as_cell(), Style::default().on_dark_gray())
}

pub fn readonly_params(params: &TableState<ReadonlyHeader>) -> UiTableGrid<'_, '_, 2> {
    UiTableGrid::new(["Header Name".blue(), "Header Value".blue()], [0.5, 0.5])
        .with_rows(
            params
                .items()
                .iter()
                .map(|p| {
                    {
                        [
                            Span::raw(if p.key.len() > 1 { &p.key } else { "-" }),
                            Span::raw(if p.value.len() > 1 { &p.value } else { "-" }),
                        ]
                    }
                })
                .collect(),
        )
        .with_index(params.idx().as_cell(), Style::default().on_dark_gray())
}
