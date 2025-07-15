use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::Span,
    widgets::Block,
};
use tui_textarea::{CursorMove, Input, TextArea};

use crate::{
    programs::tui::{
        element_view::Painter,
        elements::{dropdown::OverlayDropdown, utils::expand, Separator},
    },
    store::models::HttpMethod,
};

use super::store::{actions::Action, PaneStore};

const METHODS: [HttpMethod; 7] = [
    HttpMethod::Get,
    HttpMethod::Post,
    HttpMethod::Put,
    HttpMethod::Patch,
    HttpMethod::Delete,
    HttpMethod::Head,
    HttpMethod::Options,
];

pub struct MethodUrlBarView {
    render_area: Rect,
    // method: HttpMethod,
    url_input: TextArea<'static>,
    dropdown: OverlayDropdown<HttpMethod>,
    show_dropdown: bool,
}

impl MethodUrlBarView {
    pub fn new() -> Self {
        let mut url_input = TextArea::default();
        url_input.set_cursor_line_style(Style::default());
        url_input.set_placeholder_text("https://");

        Self {
            render_area: Rect::default(),
            url_input,
            dropdown: OverlayDropdown::with_items(HttpMethod::Get, METHODS)
                .with_highlight_style(Style::default().on_light_blue())
                .with_style(Style::default().on_light_red()),
            show_dropdown: false,
        }
    }

    fn clean_url(&mut self) {
        self.url_input.move_cursor(CursorMove::End);
        self.url_input.delete_line_by_head();
    }
}

impl MethodUrlBarView {
    pub fn draw<'painter, 'this: 'painter, 'state: 'painter>(
        &'this self,
        painter: &mut Painter<'painter>,
        state: &'state PaneStore,
    ) {
        // painter.render(|frame| {
        //     let req = state.current_request_idx().unwrap();
        //     let is_focus = state.is_focus(super::state::ElementFocus::MethodUrlBar);

        //     let border_style = is_focus
        //         .then_some(Style::default().blue())
        //         .unwrap_or_default();

        //     let line_block = Block::bordered().border_style(border_style);

        //     let area = line_block.inner(self.render_area);
        //     frame.render_widget(line_block, self.render_area);

        //     let [method_area, left_separator_area, url_area, right_reparator_area, indicator_area] =
        //         Layout::horizontal([
        //             Constraint::Length(11),
        //             Constraint::Length(1),
        //             Constraint::Min(10),
        //             Constraint::Length(1),
        //             Constraint::Length(10),
        //         ])
        //         .areas(area);

        //     frame.render_widget(
        //         Separator::default().style(border_style),
        //         left_separator_area,
        //     );
        //     frame.render_widget(
        //         Separator::default().style(border_style),
        //         right_reparator_area,
        //     );

        //     frame.render_widget(
        //         Span::from(expand(req.method().as_ref(), " ", 10))
        //             .style(Style::new().on_light_red())
        //             .black(),
        //         method_area,
        //     );

        //     frame.render_widget(&self.url_input, url_area);
        //     frame.render_widget(
        //         Span::from(expand(
        //             if self.show_dropdown { "--" } else { "Send" },
        //             " ",
        //             10,
        //         ))
        //         .on_light_green()
        //         .black(),
        //         indicator_area,
        //     );
        // });

        if self.show_dropdown {
            painter.render_last(|frame| {
                let area = Rect {
                    x: self.render_area.left() + 1,
                    y: self.render_area.bottom() - 1,
                    width: 10,
                    height: METHODS.len() as u16,
                };
                frame.render_widget(&self.dropdown, area);
            });
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent, state: &PaneStore) -> Vec<Action> {
        let mut actions = Vec::new();
        if key.kind == KeyEventKind::Press {
            if self.show_dropdown {
                match key.code {
                    KeyCode::Enter => {
                        // let request_idx = state.idx().child_idx();
                        // collector.add(EditRequest::new(
                        //     request_idx,
                        //     RequestEditType::Method(self.dropdown.selected().clone()),
                        // ));
                        self.show_dropdown = false;
                    }
                    KeyCode::Esc => {
                        self.show_dropdown = false;
                    }
                    _ => {
                        self.dropdown.handle_key(key);
                    }
                }
            } else {
                let mut mutate_on_blur = |next: bool| {
                    let request_idx = state.current_request_idx().unwrap();

                    if next {
                        actions.push(Action::NextFocus);
                    } else {
                        actions.push(Action::PreviousFocus);
                    }

                    actions.push(Action::EditRequestMethod {
                        idx: state.current_request_idx().unwrap(),
                        method: self.dropdown.selected().clone(),
                    });

                    actions.push(Action::EditRequestUrl {
                        idx: state.current_request_idx().unwrap(),
                        url: self.url_input.lines()[0].to_string(),
                    });
                };

                match key.code {
                    KeyCode::Tab => {
                        mutate_on_blur(true);
                    }
                    KeyCode::BackTab => {
                        mutate_on_blur(false);
                    }
                    KeyCode::Enter => {
                        // Change to pending
                        if key.modifiers == KeyModifiers::ALT {
                            self.show_dropdown = true;
                        }
                    }

                    _ => {
                        let key_input = Input::from(key);
                        self.url_input.input(key_input);
                    }
                }
            }
        }

        actions
    }

    pub fn set_render_area(&mut self, area: Rect) {
        self.render_area = area;
    }
}
