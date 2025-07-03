use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::Span,
};

use crate::{
    programs::tui::{element_view::ElementView, views::dashboard::editor::TextEditor},
    store::models::BodyContent,
};

pub struct BodyEditor {
    render_area: Rect,
    text_editor: TextEditor,
    pub editable: bool,
}

impl BodyEditor {
    pub fn new(editable: bool) -> Self {
        Self {
            render_area: Rect::default(),
            text_editor: TextEditor::new(editable),
            editable,
        }
    }

    pub fn draw_header_line(
        line_area: Rect,
        frame: &mut ratatui::Frame,
        body_content: &BodyContent,
    ) {
        let title = Span::from(format!(" Type: {} ", body_content.as_tag())).italic();
        let body_type = Span::from("\u{25bc} ");

        let [title_area, body_type_area] = Layout::horizontal([
            Constraint::Length(title.width() as u16),
            Constraint::Length(body_type.width() as u16),
        ])
        .flex(ratatui::layout::Flex::SpaceBetween)
        .areas(line_area);

        frame.render_widget(title, title_area);
        frame.render_widget(body_type, body_type_area);
        frame
            .buffer_mut()
            .set_style(line_area, Style::default().on_dark_gray());
    }
}

impl ElementView for BodyEditor {
    type State = BodyContent;

    fn draw(&self, frame: &mut ratatui::Frame, state: &Self::State) {
        let content_area = {
            if self.editable {
                let [header_area, content_area] =
                    Layout::vertical([Constraint::Length(1), Constraint::Fill(1)])
                        .areas(self.render_area);

                Self::draw_header_line(header_area, frame, state);
                content_area
            } else {
                self.render_area
            }
        };

        match state {
            BodyContent::Empty => {
                let text = Span::from("No body").italic();
                let [inner_area] = Layout::vertical([Constraint::Length(1)])
                    .flex(ratatui::layout::Flex::Center)
                    .areas(content_area);
                let [inner_area] = Layout::horizontal([Constraint::Length(text.width() as u16)])
                    .flex(ratatui::layout::Flex::Center)
                    .areas(inner_area);

                frame.render_widget(text, inner_area);
            }
            BodyContent::File(path_buf) => {
                let text = Span::from(path_buf.as_os_str().to_str().unwrap()).italic();
                let [inner_area] = Layout::vertical([Constraint::Length(1)])
                    .flex(ratatui::layout::Flex::Center)
                    .areas(content_area);
                let [inner_area] = Layout::horizontal([Constraint::Length(text.width() as u16)])
                    .flex(ratatui::layout::Flex::Center)
                    .areas(inner_area);

                frame.render_widget(text, inner_area);
            }
            BodyContent::Form(_hash_map) => {
                frame.render_widget(Span::from("Form values!"), content_area);
            }
            BodyContent::Text(_) => {
                frame.render_widget(&self.text_editor, content_area);
            }
        }
    }

    fn on_key(&mut self, key: crossterm::event::KeyEvent, _state: &mut Self::State) {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Enter => {
                    if key.modifiers == KeyModifiers::ALT {
                        // open menu
                    }
                }
                _ => {}
            }
        }
    }

    fn set_area(&mut self, area: Rect) {
        self.render_area = area;
    }
}
