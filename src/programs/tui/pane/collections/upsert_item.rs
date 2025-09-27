use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Block, Clear},
    Frame,
};

use crate::programs::tui::{
    common::{input::mode_input::ModeInput, Interactive, UiComposedElement, UiElementV2},
    config::Config,
};

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
    input: ModeInput,
    method_type: UpsertMethod,
    render_area: Rect,
}

impl UpsertItemPopup {
    pub fn new() -> Self {
        let upsert_method = UpsertMethod::CreateRequest;
        Self {
            input: ModeInput::new(""),
            method_type: upsert_method,
            render_area: Rect::default(),
        }
    }

    pub fn method_type(&self) -> UpsertMethod {
        self.method_type
    }

    pub fn text(&self) -> String {
        self.input.txt().to_string()
    }

    pub fn set_state(&mut self, method_type: UpsertMethod, text: &str) {
        self.method_type = method_type;
        self.input.clear();
        self.input.replace(text);
    }
}

impl<'params> UiComposedElement<'params> for UpsertItemPopup {
    type Params = &'params Config;

    fn set_area(&mut self, area: Rect, _viewport_area: Rect) {
        self.render_area = area;
        self.input.set_visual_width(area.width.saturating_sub(2));
    }

    fn draw(&self, config: Self::Params, frame: &mut Frame) {
        frame.render_widget(Clear, self.render_area);

        let block = Block::bordered()
            .title(format!("| {} |", self.method_type.as_title()))
            .border_style(Style::default().fg(config.theme.border_focus))
            .border_type(ratatui::widgets::BorderType::Thick);

        let input_area = block.inner(self.render_area);
        frame.render_widget(block, self.render_area);

        self.input.draw(input_area, frame);
    }
}

impl<'params> Interactive<'params> for UpsertItemPopup {
    type Effect = ();

    type Params = ();

    fn handle_key(
        &mut self,
        _params: Self::Params,
        key: crossterm::event::KeyEvent,
    ) -> Self::Effect {
        self.input.handle_key(key);
    }
}
