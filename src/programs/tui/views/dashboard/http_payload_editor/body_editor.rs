use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::Span,
};

use crate::{
    programs::tui::{
        element_view::ElementView,
        elements::dropdown::SharedDropdown,
        views::dashboard::{
            body_type_selector::BodyType, editor::TextEditor,
            pane_state::history::MutationCollector,
        },
    },
    store::models::BodyContent,
};

pub enum BodyEditorContent {
    Mutable(SharedDropdown<BodyType>),
    Static(BodyContent),
}

pub struct BodyEditor {
    state_content: BodyEditorContent,
    render_area: Rect,
    text_editor: TextEditor,
    // pub editable: bool,
}

impl BodyEditor {
    pub fn new_editable(shared: SharedDropdown<BodyType>) -> Self {
        Self {
            state_content: BodyEditorContent::Mutable(shared),
            render_area: Rect::default(),
            text_editor: TextEditor::new(true),
        }
    }

    pub fn new_readable() -> Self {
        Self {
            state_content: BodyEditorContent::Static(BodyContent::Empty),
            render_area: Rect::default(),
            text_editor: TextEditor::new(true),
        }
    }

    pub fn is_editable(&self) -> bool {
        match self.state_content {
            BodyEditorContent::Mutable(_) => true,
            BodyEditorContent::Static(_) => false,
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

impl<'a> ElementView<'a> for BodyEditor {
    type State = BodyContent;
    type Collector = MutationCollector<'a>;

    fn draw(&self, frame: &mut ratatui::Frame, state: &Self::State) {
        let content_area = {
            if self.is_editable() {
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

    fn on_key(&mut self, key: crossterm::event::KeyEvent, _state: &mut Self::Collector) {
        match key.code {
            KeyCode::Enter => {
                if key.modifiers == KeyModifiers::ALT {
                    if let BodyEditorContent::Mutable(dropdown_data) = &mut self.state_content {
                        // dropdown_data.borrow_mut().select(BodyType::from(_state));
                        // dropdown_data.borrow_mut().set_area(Rect {
                        //     x: self.render_area.left(),
                        //     y: self.render_area.top() + 1,
                        //     width: self.render_area.width,
                        //     height: 4,
                        // });
                    }
                }
            }
            _ => {}
        }
    }

    fn set_area(&mut self, area: Rect) {
        self.render_area = area;
    }
}
