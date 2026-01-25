use crossterm::event::KeyEvent;
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::Line,
    Frame,
};

use crate::programs::tui_v2::common::elements::ui_block;

use super::state::{CollectionsList, ListIdx, PaneState, SectionFocus, UpsertItemAction};
mod list;
pub mod upsert_item;

pub struct CollectionsSidebar {}

impl CollectionsSidebar {
    pub fn new() -> Self {
        Self {}
    }
}

impl CollectionsSidebar {
    pub fn draw(&self, area: Rect, frame: &mut Frame, state: &PaneState) {
        let block = ui_block(
            format!(" Collections ({}) ", state.collections.size()),
            false,
        );

        let inner_area = block.inner(area);

        frame.render_widget(block, area);

        let lines = page_list(&state.collections, inner_area.height);

        let mut line_area = Rect {
            height: 1,
            ..inner_area
        };

        for line in lines {
            frame.render_widget(line, line_area);
            line_area.y += 1;
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent, state: &mut PaneState) {
        match key.code {
            crossterm::event::KeyCode::Backspace
            | crossterm::event::KeyCode::Delete
            | crossterm::event::KeyCode::Char('d') => {}
            crossterm::event::KeyCode::Left | crossterm::event::KeyCode::Char('h') => {
                if let ListIdx::Group(i) = state.collections.idx {
                    state.collections.items[i].is_open = false;
                }
            }
            crossterm::event::KeyCode::Right | crossterm::event::KeyCode::Char('l') => {
                if let ListIdx::Group(i) = state.collections.idx {
                    state.collections.items[i].is_open = true;
                }
            }
            crossterm::event::KeyCode::Up | crossterm::event::KeyCode::Char('k') => {
                match state.collections.idx {
                    ListIdx::None => {}
                    ListIdx::Group(i) => {
                        if i > 0 {
                            let prev = &state.collections.items[i - 1];
                            if !prev.is_open || prev.requests.is_empty() {
                                state.collections.idx = ListIdx::Group(i - 1);
                            } else {
                                state.collections.idx =
                                    ListIdx::Item(i - 1, prev.requests.len() - 1);
                            }
                        }
                    }
                    ListIdx::Item(i, sub_i) => {
                        if sub_i == 0 {
                            state.collections.idx = ListIdx::Group(i);
                        } else {
                            state.collections.idx = ListIdx::Item(i, sub_i - 1);
                        }
                    }
                }
            }
            crossterm::event::KeyCode::Down | crossterm::event::KeyCode::Char('j') => {
                match state.collections.idx {
                    ListIdx::None => {}
                    ListIdx::Group(i) => {
                        let itm = &state.collections.items[i];
                        if itm.is_open && !itm.requests.is_empty() {
                            state.collections.idx = ListIdx::Item(i, 0);
                        } else if i < state.collections.items.len() - 1 {
                            state.collections.idx = ListIdx::Group(i + 1);
                        }
                    }
                    ListIdx::Item(i, sub_i) => {
                        let itm = &state.collections.items[i];
                        if sub_i < itm.requests.len() - 1 {
                            state.collections.idx = ListIdx::Item(i, sub_i + 1);
                        } else if i < state.collections.items.len() - 1 {
                            state.collections.idx = ListIdx::Group(i + 1);
                        }
                    }
                }
            }
            crossterm::event::KeyCode::Home => {
                if !state.collections.items.is_empty() {
                    state.collections.idx = ListIdx::Group(0);
                }
            }
            crossterm::event::KeyCode::End => {
                if let Some(itm) = state.collections.items.last() {
                    if itm.requests.is_empty() {
                        state.collections.idx = ListIdx::Group(state.collections.items.len() - 1);
                    } else {
                        state.collections.idx = ListIdx::Item(
                            state.collections.items.len() - 1,
                            itm.requests.len() - 1,
                        );
                    }
                }
            }
            crossterm::event::KeyCode::Tab => {
                state.focus = SectionFocus::UpsertItem;
            }
            crossterm::event::KeyCode::BackTab => {}
            crossterm::event::KeyCode::Char(ch) => match ch {
                'e' => match state.collections.idx {
                    ListIdx::None => {}
                    ListIdx::Group(_) => {
                        state.focus = SectionFocus::UpsertItem;
                        state.upsert_item_action = UpsertItemAction::EditCollection;
                    }
                    ListIdx::Item(_, _) => {
                        state.focus = SectionFocus::UpsertItem;
                        state.upsert_item_action = UpsertItemAction::EditRequest;
                    }
                },
                'a' => {
                    state.focus = SectionFocus::UpsertItem;
                    state.upsert_item_action = UpsertItemAction::CreateRequest;
                }
                'n' => {
                    state.focus = SectionFocus::UpsertItem;
                    state.upsert_item_action = UpsertItemAction::CreateCollection;
                }

                _ => {}
            },
            // crossterm::event::KeyCode::Delete => todo!(),
            // crossterm::event::KeyCode::Enter => todo!(),
            _ => {}
        }
    }
}

fn page_list<'a>(list: &'a CollectionsList, height: u16) -> Vec<Line<'a>> {
    let mut lines = Vec::new();

    if list.idx == ListIdx::None {
        return lines;
    }

    let flat_idx = compute_flat_idx(list);
    let start_idx = ((((flat_idx as f64) / (height as f64)).floor() as u16) * height) as usize;
    let end_idx = start_idx + (height as usize);

    let select_style = Style::new().on_dark_gray();
    let normal_style = Style::new();

    // Naive implementation :(
    let mut iterate_idx = 0;
    for coll in &list.items {
        if iterate_idx >= start_idx && iterate_idx <= end_idx {
            let symbol = if coll.is_open {
                "\u{25bc} "
            } else {
                "\u{25b6} "
            };
            lines.push(Line::default().spans([symbol, &coll.name]).style(
                if flat_idx == iterate_idx {
                    select_style
                } else {
                    normal_style
                },
            ));
        }

        iterate_idx += 1;

        if coll.is_open {
            for req in &coll.requests {
                if iterate_idx >= start_idx && iterate_idx <= end_idx {
                    lines.push(Line::default().spans(["  ", &req.name]).style(
                        if flat_idx == iterate_idx {
                            select_style
                        } else {
                            normal_style
                        },
                    ));
                }
                iterate_idx += 1;
            }
        }
        if iterate_idx >= end_idx {
            break;
        }
    }

    lines
}

fn compute_flat_idx(list: &CollectionsList) -> usize {
    let mut flat_idx = 0;
    for (coll_i, coll) in list.items.iter().enumerate() {
        match list.idx {
            ListIdx::Group(i) if i == coll_i => break,
            ListIdx::Item(i, sub_i) if i == coll_i => {
                flat_idx += sub_i + 1;
                break;
            }
            _ => {}
        }

        flat_idx += 1;
        if coll.is_open {
            flat_idx += coll.requests.len();
        }
    }

    flat_idx
}
