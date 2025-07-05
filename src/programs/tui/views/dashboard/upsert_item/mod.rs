use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, Clear},
    Frame,
};
use tui_textarea::{CursorMove, Input, TextArea};
pub use upsert_item_state::UpsertMethod;

use crate::{
    programs::tui::element_view::ElementView,
    store::models::{CollectionsModel, RequestModel},
};

use super::{global_pane_state::GlobalPaneState, pane_state::history::MutationCollector};

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
    render_area: Rect,
    input: TextArea<'static>,
}

impl UpsertItemView {
    pub fn new() -> Self {
        let mut input = TextArea::default();
        // TODO: use the state title
        input.set_block(Block::bordered().title(" Create request "));
        input.set_cursor_line_style(Style::default());

        Self {
            render_area: Rect::default(),
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
}

impl<'a> ElementView<'a> for UpsertItemView {
    type State = GlobalPaneState;
    type Collector = MutationCollector<'a>;

    fn draw(&self, frame: &mut Frame, _state: &Self::State) {
        frame.render_widget(Clear, self.render_area);
        frame.render_widget(&self.input, self.render_area);
    }

    fn set_area(&mut self, area: ratatui::prelude::Rect) {
        let _area = {
            let [area] = Layout::vertical([Constraint::Length(3)])
                .flex(ratatui::layout::Flex::Center)
                .areas(area);

            let [area] = Layout::horizontal([Constraint::Percentage(40)])
                .flex(ratatui::layout::Flex::Center)
                .areas(area);

            area
        };
        self.render_area = _area;
    }

    fn on_key(&mut self, key: KeyEvent, collecor: &mut Self::Collector) {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Enter => {
                    let name = self.input.lines()[0].clone();
                    // match collecor.upsert_method() {
                    //     UpsertMethod::CreateRequest(_) => {
                    //         collecor.add_collection_request(RequestModel::new(name));
                    //     }
                    //     UpsertMethod::CreateCollection(_) => {
                    //         collecor.add_collection(CollectionsModel::new(name));
                    //     }
                    //     UpsertMethod::EditRequest(_) | UpsertMethod::EditCollection(_) => {
                    //         collecor.edit_item_name(name);
                    //     }
                    // }

                    self.reset();
                    // collecor.hidden_overlay();
                    // collecor.set_focus(super::focus::ElementFocus::Collections);
                }
                KeyCode::Esc => {
                    self.reset();
                    // collecor.hidden_overlay();
                }
                _ => {
                    let input = Input::from(key);
                    self.input.input(input);
                }
            }
        }
    }

    fn on_change_state(&mut self, _state: &Self::State) {}
}
