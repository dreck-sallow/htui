use crossterm::event::KeyEvent;
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

use super::state::{ListIdx, PaneState, ParamsTable, SectionFocus};

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
}

impl RequestBuilder {
    pub fn new() -> Self {
        Self {
            section: Section::Params,
        }
    }

    fn tab_idx(&self) -> usize {
        match self.section {
            Section::Params => 0,
            Section::Headers => 1,
            Section::Body => 2,
        }
    }

    fn next_tab(&mut self) {
        self.section = match self.section {
            Section::Params => Section::Headers,
            Section::Headers => Section::Body,
            Section::Body => Section::Params,
        };
    }

    fn prev_tab(&mut self) {
        self.section = match self.section {
            Section::Params => Section::Body,
            Section::Headers => Section::Params,
            Section::Body => Section::Headers,
        };
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

        let tabs = Tabs::new([
            Section::Params.as_str(),
            Section::Headers.as_str(),
            Section::Body.as_str(),
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
                    Style::default().fg(is_focus.then_some(Color::Blue).unwrap_or(Color::DarkGray)),
                ),
        );

        frame.render_widget(tabs, tabs_area);

        // render content
        if let ListIdx::Item(i, sub_i) = state.collections.idx {
            let request = &state.collections.items[i].requests[sub_i];
            match self.section {
                Section::Params => {
                    params_to_ui(&request.params).draw(content_area, frame);
                }
                Section::Headers => params_to_ui(&request.headers).draw(content_area, frame),
                Section::Body => {}
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent, state: &mut PaneState) {
        match self.section {
            Section::Params => self.handle_params_key(key),
            Section::Headers => self.handle_headers_key(key),
            Section::Body => self.handle_body_key(key),
        }
    }

    fn handle_params_key(&mut self, key: KeyEvent) {}

    fn handle_headers_key(&mut self, key: KeyEvent) {}

    fn handle_body_key(&mut self, key: KeyEvent) {}
}

fn params_to_ui(params: &ParamsTable) -> UiTableGrid<'_, '_, 3> {
    UiTableGrid::new(
        [
            "Enable".on_blue(),
            "Header Name".on_blue(),
            "Header Value".on_blue(),
        ],
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
                        Span::raw(&p.key),
                        Span::raw(&p.value),
                    ]
                }
            })
            .collect(),
    )
    .with_index(params.idx, Style::default().on_dark_gray())
}
