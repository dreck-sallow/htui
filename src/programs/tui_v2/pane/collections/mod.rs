use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::Line,
    Frame,
};

use crate::programs::tui_v2::common::elements::ui_block;

use super::state::{CollectionsList, ListIdx, PaneState};
mod list;
mod upsert_item;

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

    pub fn draw_last(&self, area: Rect, frame: &mut Frame, state: &PaneState) {}
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
