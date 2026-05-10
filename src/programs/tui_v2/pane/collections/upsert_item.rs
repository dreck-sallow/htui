use crossterm::event::KeyEvent;
use ratatui::Frame;

use crate::programs::tui_v2::{
    common::{
        elements::{ui_block, utils::center_area},
        input::input_mode::InputMode,
    },
    pane::state_v2::{
        collections::{CollectionItem, ListIdx, RequestItem},
        PaneState,
    },
};

#[derive(PartialEq, Eq)]
pub enum UpsertItemAction {
    CreateCollection,
    CreateRequest,
    EditCollection,
    EditRequest,
}

pub struct UpertItemPopup {
    input: InputMode,
    action: UpsertItemAction,
    pub(crate) show_overlay: bool,
}

impl UpertItemPopup {
    pub fn new() -> Self {
        Self {
            input: InputMode::new(""),
            action: UpsertItemAction::CreateCollection,
            show_overlay: false,
        }
    }

    fn title(action: &UpsertItemAction) -> &'static str {
        match action {
            UpsertItemAction::CreateCollection => " Create Collection ",
            UpsertItemAction::CreateRequest => " Create Request ",
            UpsertItemAction::EditCollection => " Edit Collection ",
            UpsertItemAction::EditRequest => " Edit Request ",
        }
    }

    pub fn open(&mut self, action: UpsertItemAction, initial: &str) {
        self.action = action;
        self.input.reset();
        self.input.replace(initial);
        self.input.move_end_cursor();
        self.show_overlay = true;
    }

    pub fn draw(&self, frame: &mut Frame, _state: &PaneState) {
        let area = center_area(
            frame.area(),
            ratatui::layout::Constraint::Length(3),
            ratatui::layout::Constraint::Percentage(30),
        );
        let block = ui_block(Self::title(&self.action), true);
        let inner_area = block.inner(area);

        frame.render_widget(block, area);
        self.input.draw(inner_area, frame);
    }

    pub fn handle_key(&mut self, key: KeyEvent, state: &mut PaneState) {
        match key.code {
            crossterm::event::KeyCode::Esc if !self.input.is_editing() => {
                // cancel
                self.input.reset();
                self.show_overlay = false;
            }
            crossterm::event::KeyCode::Enter => {
                let input = self.input.inner().to_string();
                match self.action {
                    UpsertItemAction::CreateCollection => {
                        match state.collections.idx {
                            ListIdx::None => {
                                state.collections.add_item(CollectionItem::new(input));
                                state.collections.idx = ListIdx::Group(0);
                            }
                            ListIdx::Group(i) | ListIdx::Item(i, _) => {
                                state
                                    .collections
                                    .insert_item(i + 1, CollectionItem::new(input));
                            }
                        };
                    }
                    UpsertItemAction::CreateRequest => {
                        match state.collections.idx {
                            ListIdx::None => {}
                            ListIdx::Group(i) => {
                                state
                                    .collections
                                    .insert_req((i, 0), RequestItem::new(input));
                            }
                            ListIdx::Item(i, sub_i) => {
                                state
                                    .collections
                                    .insert_req((i, sub_i + 1), RequestItem::new(input));
                            }
                        };
                    }
                    UpsertItemAction::EditCollection => {
                        if let ListIdx::Group(i) = state.collections.idx {
                            state.collections.get_mut_collection(i).unwrap().name = input;
                        }
                    }
                    UpsertItemAction::EditRequest => {
                        if let ListIdx::Item(i, sub_i) = state.collections.idx {
                            state.collections.get_mut_requests(i).unwrap()[sub_i].name = input;
                        }
                    }
                }

                // Submit
                self.input.reset();
                self.show_overlay = false;
            }
            _ => {
                self.input.handle_key(key);
            }
        }
    }
}
