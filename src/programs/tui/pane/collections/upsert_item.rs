use std::rc::Rc;

use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    widgets::Block,
    Frame,
};
use tui_textarea::{CursorMove, Input, TextArea};

use crate::programs::tui::config::Config;

#[derive(Clone, Copy)]
pub enum UpsertMethod {
    CreateRequest,
    CreateCollection,
    EditRequest,
    EditCollection,
}

impl UpsertMethod {
    pub fn as_title(&self) -> &'static str {
        match self {
            UpsertMethod::CreateRequest => "Create request",
            UpsertMethod::CreateCollection => "Create collection",
            UpsertMethod::EditRequest => "Edit request",
            UpsertMethod::EditCollection => "Edit collection",
        }
    }
}

pub struct UpsertItemPopup {
    input: TextArea<'static>,
    method_type: UpsertMethod,
    config: Rc<Config>,
}

impl UpsertItemPopup {
    pub fn new(config: Rc<Config>) -> Self {
        let upsert_method = UpsertMethod::CreateRequest;

        let mut input = TextArea::default();
        input.set_block(
            Block::bordered()
                .title(format!("| {} |", upsert_method.as_title()))
                .border_style(Style::default().fg(config.theme.border_focus))
                .border_type(ratatui::widgets::BorderType::Thick),
        );
        input.set_cursor_line_style(Style::default());

        Self {
            input,
            method_type: upsert_method,
            config,
        }
    }

    pub fn method_type(&self) -> UpsertMethod {
        self.method_type
    }

    pub fn text(&self) -> String {
        self.input.lines()[0].to_string()
    }

    fn clean_input(&mut self) {
        self.input.move_cursor(CursorMove::End);
        self.input.delete_line_by_head();
    }

    pub fn set_state(&mut self, method_type: UpsertMethod, text: &str) {
        self.method_type = method_type;
        self.clean_input();
        self.input.insert_str(text);

        self.input.set_block(
            Block::bordered()
                .title(format!("| {} |", self.method_type.as_title()))
                .border_style(Style::default().fg(self.config.theme.border_focus))
                .border_type(ratatui::widgets::BorderType::Thick),
        );
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(&self.input, area);
    }

    pub fn handle_input(&mut self, input: Input) {
        self.input.input(input); // XD
    }
}
