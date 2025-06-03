use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout},
    style::Style,
    widgets::Block,
    Frame,
};
use tui_textarea::{CursorMove, Input, TextArea};
use upsert_item_state::UpsertItemState;
pub use upsert_item_state::UpsertMethod;

use super::{action::Action, pane_state::PaneState};

mod upsert_item_state;

fn method_to_title(method: &UpsertMethod) -> &'static str {
    match method {
        UpsertMethod::CreateRequest => " Create Request ",
        UpsertMethod::CreateCollection => " Create collection ",
        UpsertMethod::EditRequest => " Edit request ",
        UpsertMethod::EditCollection => " Edit collection ",
    }
}

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

    fn set_method(&mut self, method: UpsertMethod, text: &str) {
        self.input
            .set_block(Block::bordered().title(method_to_title(&method)));
        self.state.set_method(method);
        self.input.insert_str(text);
    }

    // pub fn edit_collection(&mut self) {
    //     self.set_method(UpsertMethod::EditCollection, " Edit collection ");
    // }

    // pub fn edit_request(&mut self) {
    //     self.set_method(UpsertMethod::EditRequest, " Edit request ");
    // }

    // pub fn create_collection(&mut self, text: String) {
    //     self.set_method(UpsertMethod::CreateCollection, " Create collection ");
    //     self.input.insert_str(text);
    // }

    // pub fn create_request(&mut self, text: String) {
    //     self.set_method(UpsertMethod::CreateRequest, " Create request ");
    //     self.input.insert_str(text);
    // }

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

    pub fn handle_key(
        &mut self,
        key: KeyEvent,
        state: &mut PaneState,
        action_register: &mut Vec<Action>,
    ) {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Enter => {
                    let method = self.state.method();
                    action_register.push(Action::SaveUpsertItem(
                        method,
                        self.input.lines()[0].clone(),
                    ));

                    self.reset();
                    state.hidden_overlay();
                    state.focus_element(super::focus::ElementFocus::Collections);
                }
                KeyCode::Esc => {
                    self.reset();
                    state.focus_element(super::focus::ElementFocus::Collections);
                    state.hidden_overlay();
                }
                _ => {
                    let input = Input::from(key);
                    self.input.input(input);
                }
            }
        }
    }

    pub fn handle_action(&mut self, action: Action) {
        match action {
            Action::UpsertItem(upsert_method, title) => {
                self.set_method(upsert_method, &title);
            }
            _ => {}
        }
    }
}
