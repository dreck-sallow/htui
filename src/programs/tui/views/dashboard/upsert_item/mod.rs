use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout},
    style::Style,
    widgets::Block,
    Frame,
};
use tui_textarea::{CursorMove, Input, TextArea};
use upsert_item_state::{UpsertItemState, UpsertMethod};

mod upsert_item_state;

pub struct UpsertItemView {
    state: UpsertItemState,
    input: TextArea<'static>,
}

impl UpsertItemView {
    pub fn new() -> Self {
        let mut input = TextArea::default();
        // TODO: use the state title
        input.set_block(Block::bordered().title(" Create request "));
        input.set_cursor_line_style(Style::default());

        Self {
            state: UpsertItemState::new(),
            input,
        }
    }

    fn reset(&mut self) {
        self.state.reset();
        self.input.move_cursor(CursorMove::End);
        self.input.delete_line_by_head();
    }

    fn set_method(&mut self, method: UpsertMethod, title: &'static str) {
        self.state.set_method(method);
        self.input.set_block(Block::bordered().title(title));
    }

    pub fn edit_collection(&mut self) {
        self.set_method(UpsertMethod::EditCollection, " Edit collection ");
    }

    pub fn edit_request(&mut self) {
        self.set_method(UpsertMethod::EditRequest, " Edit request ");
    }

    pub fn create_collection(&mut self, text: String) {
        self.set_method(UpsertMethod::CreateCollection, " Create collection ");
        self.input.insert_str(text);
    }

    pub fn create_request(&mut self, text: String) {
        self.set_method(UpsertMethod::CreateRequest, " Create request ");
        self.input.insert_str(text);
    }

    pub fn draw(&self, frame: &mut Frame) {
        let area = {
            let [area] = Layout::vertical([Constraint::Length(3)])
                .flex(ratatui::layout::Flex::Center)
                .areas(frame.area());

            let [area] = Layout::horizontal([Constraint::Percentage(40)])
                .flex(ratatui::layout::Flex::Center)
                .areas(area);

            area
        };

        frame.render_widget(&self.input, area);
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Enter => {
                    self.reset();
                }
                KeyCode::Esc => {
                    self.reset();
                }
                _ => {
                    let input = Input::from(key);
                    self.input.input(input);
                }
            }
        }
    }
}
