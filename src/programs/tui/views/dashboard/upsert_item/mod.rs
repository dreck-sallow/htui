use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Style, Stylize},
    widgets::Block,
    Frame,
};
use tui_textarea::{CursorMove, Input, TextArea};
use upsert_item_state::UpsertItemState;
pub use upsert_item_state::UpsertMethod;

use crate::store::models::{CollectionsModel, RequestModel};

use super::{action::Action, global_pane_state::GlobalPaneState, pane_state::PaneState};

pub mod upsert_item_state;

fn method_to_title(method: &UpsertMethod) -> &'static str {
    match method {
        UpsertMethod::CreateRequest(_) => " Create Request ",
        UpsertMethod::CreateCollection(_) => " Create collection ",
        UpsertMethod::EditRequest(_) => " Edit request ",
        UpsertMethod::EditCollection(_) => " Edit collection ",
    }
}

pub struct UpsertItemView {
    // state: UpsertItemState,
    input: TextArea<'static>,
}

impl UpsertItemView {
    pub fn new() -> Self {
        let mut input = TextArea::default();
        // TODO: use the state title
        input.set_block(Block::bordered().title(" Create request "));
        input.set_cursor_line_style(Style::default());

        Self {
            // state: UpsertItemState::new(),
            input,
        }
    }

    fn reset(&mut self) {
        // self.state.reset();
        self.input.move_cursor(CursorMove::End);
        self.input.delete_line_by_head();
    }

    fn set_method(&mut self, method: UpsertMethod) {
        self.input.set_block(
            Block::bordered()
                .title(method_to_title(&method))
                .border_style(Style::default().blue()),
        );
        // self.state.set_method(method);
        self.input.insert_str(method.text());
    }

    pub fn set_inner(&mut self, state: &GlobalPaneState) {
        let method = state.upsert_method();
        self.set_method(method);
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

    pub fn handle_key(&mut self, key: KeyEvent, state: &mut GlobalPaneState) {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Enter => {
                    let name = self.input.lines()[0].clone();
                    match state.upsert_method() {
                        UpsertMethod::CreateRequest(_) => {
                            state.add_collection_request(RequestModel::new(name));
                        }
                        UpsertMethod::CreateCollection(_) => {
                            state.add_collection(CollectionsModel::new(name));
                        }
                        UpsertMethod::EditRequest(_) | UpsertMethod::EditCollection(_) => {
                            state.edit_item_name(name);
                        }
                    }

                    self.reset();
                    state.hidden_overlay();
                    state.set_focus(super::focus::ElementFocus::Collections);
                }
                KeyCode::Esc => {
                    self.reset();
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
        // match action {
        //     Action::UpsertItem(upsert_method, title) => {
        //         self.set_method(upsert_method, &title);
        //     }
        //     _ => {}
        // }
    }
}
