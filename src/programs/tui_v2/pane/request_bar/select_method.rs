use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Layout, Margin, Rect},
    style::{Style, Stylize},
    symbols,
    text::Span,
    Frame,
};

use crate::{
    programs::tui_v2::{
        common::{
            elements::{ui_block, ui_placeholder, utils::center_area},
            input::input_mode::InputMode,
            select_list::SelectList,
        },
        pane::state_v2::PaneState,
    },
    store::models::HttpMethod,
};

const LIST: [HttpMethod; 7] = [
    HttpMethod::Get,
    HttpMethod::Post,
    HttpMethod::Put,
    HttpMethod::Patch,
    HttpMethod::Delete,
    HttpMethod::Head,
    HttpMethod::Options,
];

#[derive(PartialEq, Eq)]
enum Section {
    Standard,
    Custom,
}

pub struct SelectMethodPopup {
    section: Section,
    custom_input: InputMode,
    list: SelectList<HttpMethod>,
}

impl SelectMethodPopup {
    pub fn new() -> Self {
        Self {
            section: Section::Standard,
            custom_input: InputMode::new("").with_placeholder(ui_placeholder("Method")),
            list: SelectList::new(&LIST),
        }
    }

    pub fn reset(&mut self) {
        self.section = Section::Standard;
        self.custom_input.reset();
        self.list.reset();
    }

    pub fn select(&mut self, method: &HttpMethod) {
        if let HttpMethod::Custom(s) = method {
            self.section = Section::Custom;
            self.custom_input.replace(s);
        } else {
            self.section = Section::Standard;
            self.list.select(method);
        }
    }

    pub fn is_editing(&self) -> bool {
        self.custom_input.is_editing()
    }

    pub fn selected(&self) -> Option<HttpMethod> {
        match self.section {
            Section::Standard => self.list.selected().cloned(),
            Section::Custom => {
                let st = self.custom_input.inner();
                if st.chars().count() > 0 {
                    Some(HttpMethod::Custom(st.to_string()))
                } else {
                    None
                }
            }
        }
    }

    pub fn draw(&self, frame: &mut Frame) {
        let area = center_area(frame.area(), Constraint::Length(7), Constraint::Length(56));

        let block = ui_block(" Select http method or define your own ", true);

        let [list_area, middle_area, create_area] = {
            let [left, right] = Layout::horizontal([Constraint::Fill(50), Constraint::Fill(50)])
                .areas(block.inner(Rect {
                    width: area.width - 1,
                    ..area
                }));

            [
                left,
                Rect {
                    x: left.right() + 1,
                    width: 1,
                    ..left
                },
                Rect {
                    x: right.x + 2,
                    width: right.width - 1,
                    ..right
                },
            ]
        };
        frame.render_widget(block, area);

        let (title_area, list_area) = to_section_area(list_area);
        frame.render_widget(
            Span::raw(format!("Standard ({})", LIST.len()))
                .style(self.section_style(Section::Standard)),
            title_area,
        );
        self.list.draw(list_area, frame);

        // render middle line
        for y in middle_area.top()..middle_area.bottom() {
            frame.buffer_mut().set_string(
                middle_area.x,
                y,
                symbols::line::VERTICAL,
                Style::new().blue(),
            );
        }

        // draw custom method
        let (title_area, input_area) = to_section_area(create_area);
        frame.render_widget(
            Span::raw("Custom").style(self.section_style(Section::Custom)),
            title_area,
        );
        self.custom_input.draw(input_area, frame);
    }

    fn section_style(&self, section: Section) -> Style {
        if self.section == section {
            Style::default().underlined().blue().bold()
        } else {
            Style::default()
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent, _state: &mut PaneState) {
        match self.section {
            Section::Standard => match key.code {
                crossterm::event::KeyCode::Right => {
                    self.section = Section::Custom;
                }
                _ => {
                    self.list.handle_key(key);
                }
            },
            Section::Custom => match key.code {
                crossterm::event::KeyCode::Left if !self.custom_input.is_editing() => {
                    self.section = Section::Standard;
                }
                _ => {
                    self.custom_input.handle_key(key);
                }
            },
        }
    }
}

fn to_section_area(area: Rect) -> (Rect, Rect) {
    let area = area.inner(Margin::new(1, 0));
    (
        Rect {
            x: area.left(),
            height: 1,
            ..area
        },
        Rect {
            y: area.top() + 1,
            height: area.height.saturating_sub(1),
            ..area
        },
    )
}
