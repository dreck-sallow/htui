use body_editor::BodyEditor;
use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Tabs},
    Frame,
};

use crate::programs::tui_v2::{
    common::elements::{ui_block, ui_highlight},
    events::DrawSignal,
};

use super::{
    common::params_table::ParamsTableUi,
    state::{BodyContent, ListIdx, PaneState, ParamsTable, SectionFocus},
};

mod body_editor;
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
    params_table_ui: ParamsTableUi,
    body_editor: BodyEditor,
}

impl RequestBuilder {
    pub fn new(draw_signal: DrawSignal) -> Self {
        Self {
            section: Section::Headers,
            params_table_ui: ParamsTableUi::new(),
            body_editor: BodyEditor::new(draw_signal),
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
        self.params_table_ui.is_editing() || self.body_editor.is_editing()
    }

    pub fn sync(&mut self, state: &mut PaneState) {
        self.body_editor.sync(state);
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
                format!(
                    "{} ({})",
                    Section::Body.as_str(),
                    self.body_editor.type_label()
                ),
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
                    self.params_table_ui
                        .draw(&request.params, content_area, frame);
                }
                Section::Headers => {
                    self.params_table_ui
                        .draw(&request.headers, content_area, frame)
                }
                Section::Body => {
                    self.body_editor.draw(content_area, frame);
                }
            }
        }
    }

    pub fn draw_overlay(&self, frame: &mut Frame) {
        self.params_table_ui.draw_overlay(frame);
        self.body_editor.draw_overlay(frame);
    }

    pub async fn handle_key(&mut self, key: KeyEvent, state: &mut PaneState) {
        match key.code {
            crossterm::event::KeyCode::Left if !self.is_editing() => {
                self.prev_tab();
            }
            crossterm::event::KeyCode::Right if !self.is_editing() => {
                self.next_tab();
            }
            crossterm::event::KeyCode::Tab if !self.is_editing() => {
                state.focus = SectionFocus::ResponseViewer;
                self.body_editor.save_to_state(state);
            }
            crossterm::event::KeyCode::BackTab if !self.is_editing() => {
                state.focus = SectionFocus::RequestBar;
                self.body_editor.save_to_state(state);
            }
            _ => {
                if let Some(request) = state.collections.current_req_mut() {
                    match self.section {
                        Section::Headers => self.handle_table_key(key, &mut request.headers),
                        Section::Params => self.handle_table_key(key, &mut request.params),
                        Section::Body => self.handle_body_key(key, &mut request.body).await,
                    }
                }
            }
        }
    }

    fn handle_table_key(&mut self, key: KeyEvent, table: &mut ParamsTable) {
        self.params_table_ui.handle_table_key(key, table);
    }

    async fn handle_body_key(&mut self, key: KeyEvent, state: &mut BodyContent) {
        self.body_editor.handle_key(key, state).await;
    }
}
