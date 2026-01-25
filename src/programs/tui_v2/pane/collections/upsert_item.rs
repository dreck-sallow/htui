use crossterm::event::KeyEvent;
use ratatui::Frame;

use crate::programs::tui_v2::{
    common::{
        elements::{ui_block, utils::center_area},
        input::input_mode::InputMode,
    },
    pane::state::{
        CollectionItem, ListIdx, PaneState, RequestItem, SectionFocus, UpsertItemAction,
    },
};

pub struct UpertItemPopup {
    input: InputMode,
}

impl UpertItemPopup {
    pub fn new() -> Self {
        Self {
            input: InputMode::new(""),
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

    pub fn draw(&self, frame: &mut Frame, state: &PaneState) {
        let area = center_area(
            frame.area(),
            ratatui::layout::Constraint::Length(3),
            ratatui::layout::Constraint::Percentage(30),
        );
        let block = ui_block(Self::title(&state.upsert_item_action), true);
        let inner_area = block.inner(area);

        frame.render_widget(block, area);
        self.input.draw(inner_area, frame);
    }

    pub fn handle_key(&mut self, key: KeyEvent, state: &mut PaneState) {
        match key.code {
            crossterm::event::KeyCode::Esc if !self.input.is_editing() => {
                // cancel
                state.focus = SectionFocus::Collections;
                self.input.reset();
            }
            crossterm::event::KeyCode::Enter => {
                let input = self.input.inner().to_string();
                match state.upsert_item_action {
                    UpsertItemAction::CreateCollection => {
                        match state.collections.idx {
                            ListIdx::None => {
                                state.collections.items.push(CollectionItem::new(input));
                            }
                            ListIdx::Group(i) | ListIdx::Item(i, _) => {
                                state
                                    .collections
                                    .items
                                    .insert(i + 1, CollectionItem::new(input));
                            }
                        };
                    }
                    UpsertItemAction::CreateRequest => {
                        match state.collections.idx {
                            ListIdx::None => {}
                            ListIdx::Group(i) => {
                                state.collections.items[i]
                                    .requests
                                    .insert(0, RequestItem::new(input));
                            }
                            ListIdx::Item(i, sub_i) => {
                                state.collections.items[i]
                                    .requests
                                    .insert(sub_i + 1, RequestItem::new(input));
                            }
                        };
                    }
                    UpsertItemAction::EditCollection => {
                        if let ListIdx::Group(i) = state.collections.idx {
                            state.collections.items[i].name = input;
                        }
                    }
                    UpsertItemAction::EditRequest => {
                        if let ListIdx::Item(i, sub_i) = state.collections.idx {
                            state.collections.items[i].requests[sub_i].name = input;
                        }
                    }
                }

                // Submit
                state.focus = SectionFocus::Collections;
                self.input.reset();
            }
            _ => {
                self.input.handle_key(key);
            }
        }
    }
}
