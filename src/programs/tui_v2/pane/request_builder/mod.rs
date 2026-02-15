use crossterm::event::KeyEvent;
use edit_param_table::EditParamTablePopup;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Span,
    widgets::{Block, BorderType, Borders, Tabs},
    Frame,
};

use crate::programs::tui_v2::common::{
    elements::{ui_block, ui_highlight},
    table_grid::UiTableGrid,
};

use super::state::{ListIdx, PaneState, ParamItem, ParamsTable, SectionFocus};

mod edit_param_table;

#[derive(PartialEq, Eq)]
enum Section {
    Params,
    Headers,
    Body,
}

impl Section {
    pub fn as_str(&self) -> &'static str {
        match self {
            Section::Params => "Params",
            Section::Headers => "Headers",
            Section::Body => "Body",
        }
    }
}

pub struct RequestBuilder {
    section: Section,
    edit_param_popup: EditParamTablePopup,
}

impl RequestBuilder {
    pub fn new() -> Self {
        Self {
            section: Section::Headers,
            edit_param_popup: EditParamTablePopup::new(),
        }
    }

    fn tab_idx(&self) -> usize {
        match self.section {
            Section::Headers => 0,
            Section::Params => 1,
            Section::Body => 2,
        }
    }

    fn next_tab(&mut self) {
        self.section = match self.section {
            Section::Headers => Section::Params,
            Section::Params => Section::Body,
            Section::Body => Section::Headers,
        };
    }

    fn prev_tab(&mut self) {
        self.section = match self.section {
            Section::Headers => Section::Body,
            Section::Params => Section::Headers,
            Section::Body => Section::Params,
        };
    }

    fn is_editing(&self) -> bool {
        self.edit_param_popup.is_visible()
    }
}

impl RequestBuilder {
    pub fn draw(&self, area: Rect, frame: &mut Frame, state: &PaneState) {
        let is_focus = state.focus == SectionFocus::RequestBuilder;
        let block = ui_block("", is_focus);
        let inner_area = block.inner(area);

        let [tabs_area, content_area] =
            Layout::vertical([Constraint::Length(2), Constraint::Fill(1)]).areas(inner_area);

        frame.render_widget(block, area);

        // render content
        if let ListIdx::Item(i, sub_i) = state.collections.idx {
            let request = &state.collections.items[i].requests[sub_i];

            let tabs = Tabs::new([
                format!("{} ({})", Section::Headers.as_str(), request.headers.len()),
                format!("{} ({})", Section::Params.as_str(), request.params.len()),
                Section::Body.as_str().to_string(),
            ])
            .select(self.tab_idx())
            .highlight_style(ui_highlight())
            .block(
                Block::bordered()
                    .border_type(if is_focus {
                        BorderType::Thick
                    } else {
                        BorderType::Plain
                    })
                    .borders(Borders::BOTTOM)
                    .border_style(
                        Style::default()
                            .fg(is_focus.then_some(Color::Blue).unwrap_or(Color::DarkGray)),
                    ),
            );

            frame.render_widget(tabs, tabs_area);

            match self.section {
                Section::Params => {
                    params_to_ui(&request.params).draw(content_area, frame);
                }
                Section::Headers => params_to_ui(&request.headers).draw(content_area, frame),
                Section::Body => {}
            }
        }
    }

    pub fn draw_overlay(&self, frame: &mut Frame) {
        if self.edit_param_popup.is_visible() {
            self.edit_param_popup.draw(frame);
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent, state: &mut PaneState) {
        match key.code {
            crossterm::event::KeyCode::Left if !self.is_editing() => {
                self.prev_tab();
            }
            crossterm::event::KeyCode::Right if !self.is_editing() => {
                self.next_tab();
            }
            crossterm::event::KeyCode::Tab if !self.is_editing() => {
                state.focus = SectionFocus::RequestBuilder;
            }
            crossterm::event::KeyCode::BackTab if !self.is_editing() => {
                state.focus = SectionFocus::RequestBar;
            }
            _ => {
                if let Some(request) = state.collections.current_req_mut() {
                    match self.section {
                        Section::Headers => self.handle_table_key(key, &mut request.headers),
                        Section::Params => self.handle_table_key(key, &mut request.params),
                        Section::Body => self.handle_body_key(key),
                    }
                }
            }
        }
    }

    fn handle_table_key(&mut self, key: KeyEvent, table: &mut ParamsTable) {
        if self.edit_param_popup.is_visible() {
            self.edit_param_popup.handle_key(key, table);
        } else {
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
                                    self.edit_param_popup.edit_key(&itm.key);
                                }
                                2 => {
                                    self.edit_param_popup.edit_value(&itm.value);
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

    fn handle_body_key(&mut self, key: KeyEvent) {}
}

fn params_to_ui(params: &ParamsTable) -> UiTableGrid<'_, '_, 3> {
    UiTableGrid::new(
        ["Enable".blue(), "Header Name".blue(), "Header Value".blue()],
        [0.2, 0.4, 0.4],
    )
    .with_rows(
        params
            .items
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
    .with_index(params.idx, Style::default().on_dark_gray())
}
