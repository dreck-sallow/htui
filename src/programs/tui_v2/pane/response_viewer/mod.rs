use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{Block, BorderType, Borders, Tabs},
    Frame,
};

use crate::programs::tui_v2::common::elements::{ui_block, ui_highlight};

use super::state::{PaneState, SectionFocus};

enum SectionTab {
    Headers,
    Body,
}

impl SectionTab {
    pub fn label(&self) -> &'static str {
        match self {
            SectionTab::Headers => "Headers",
            SectionTab::Body => "Body",
        }
    }
}

struct Section {
    tab: SectionTab,
}

impl Section {
    pub fn new() -> Self {
        Self {
            tab: SectionTab::Body,
        }
    }

    pub fn next(&mut self) {
        self.tab = match self.tab {
            SectionTab::Headers => SectionTab::Body,
            SectionTab::Body => SectionTab::Headers,
        };
    }

    pub fn prev(&mut self) {
        // Same logic :D
        self.next();
    }

    pub fn idx(&self) -> u8 {
        match self.tab {
            SectionTab::Body => 0,
            SectionTab::Headers => 1,
        }
    }
}

pub struct ResponseViewer {
    section: Section,
}

impl ResponseViewer {
    pub fn new() -> Self {
        Self {
            section: Section::new(),
        }
    }
}

impl ResponseViewer {
    pub fn draw(&self, area: Rect, frame: &mut Frame, state: &PaneState) {
        let is_focus = state.focus == SectionFocus::ResponseViewer;
        let block = ui_block("", is_focus);

        let (line_area, tabs_area, content_area) = {
            let [line_status, tabs, content] = Layout::vertical([
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Fill(1),
            ])
            .areas(block.inner(area));
            (line_status, tabs, content)
        };

        frame.render_widget(block, area);

        //- Render line
        frame.render_widget(Span::raw("Status line"), line_area);

        //- Render tabs
        let tabs = Tabs::new([
            format!("{} ({})", SectionTab::Body.label(), 0),
            format!("{} ({})", SectionTab::Headers.label(), 0),
        ])
        .select(self.section.idx() as usize)
        .highlight_style(ui_highlight())
        .block(
            Block::bordered()
                .border_type(if is_focus {
                    BorderType::Thick
                } else {
                    BorderType::Plain
                })
                .borders(Borders::BOTTOM | Borders::TOP)
                .border_style(
                    Style::default().fg(is_focus.then_some(Color::Blue).unwrap_or(Color::DarkGray)),
                ),
        );

        frame.render_widget(tabs, tabs_area);

        //- Render content
    }

    pub fn draw_overlay(&self, frame: &mut Frame) {}

    pub fn handle_key(&mut self, key: KeyEvent, state: &mut PaneState) {
        match key.code {
            crossterm::event::KeyCode::Left => {
                self.section.next();
            }
            crossterm::event::KeyCode::Right => {
                self.section.next();
            }
            crossterm::event::KeyCode::Tab => {
                state.focus = SectionFocus::Collections;
            }
            crossterm::event::KeyCode::BackTab => {
                state.focus = SectionFocus::RequestBuilder;
            }
            _ => {
                // if let Some(request) = state.collections.current_req_mut() {
                //     match self.section {
                //         Section::Headers => self.handle_table_key(key, &mut request.headers),
                //         Section::Params => self.handle_table_key(key, &mut request.params),
                //         Section::Body => self.handle_body_key(key, &mut request.body).await,
                //     }
                // }
            }
        }
    }
}
