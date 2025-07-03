use std::{cell::RefCell, rc::Rc};

use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::Span,
    widgets::Block,
};
use tui_textarea::{Input, TextArea};

use crate::{
    programs::tui::{
        element_view::ElementView,
        elements::{
            dropdown::{OverlayDropdownData, SharedDropdown},
            utils::expand,
            Separator,
        },
    },
    store::models::HttpMethod,
};

use super::{global_pane_state::GlobalPaneState, method_selector::MethodSelectorView};

pub struct MethodUrlBarView {
    render_area: Rect,
    dropdown_data_ref: SharedDropdown<HttpMethod>,
    url_input: TextArea<'static>,
    is_sending: bool,
    request_idx: (usize, usize),
}

impl MethodUrlBarView {
    pub fn new_with_dropdown() -> (Self, MethodSelectorView) {
        let dropdown_data = Rc::new(RefCell::new(OverlayDropdownData::new(HttpMethod::Get)));

        let mut url_input = TextArea::default();
        url_input.set_cursor_line_style(Style::default());
        url_input.set_placeholder_text("Enter a url");
        url_input.insert_str("https://");

        let this = Self {
            render_area: Rect::default(),
            dropdown_data_ref: Rc::clone(&dropdown_data),
            url_input,
            is_sending: false,
            request_idx: (0, 0),
        };

        (this, MethodSelectorView::new(dropdown_data))
    }

    fn clean_url(&mut self) {
        self.url_input.move_cursor(tui_textarea::CursorMove::End);
        self.url_input.delete_line_by_head();
    }
}

impl ElementView for MethodUrlBarView {
    type State = GlobalPaneState;

    fn draw(&self, frame: &mut ratatui::Frame, state: &Self::State) {
        let request = state.current_request().unwrap();
        let is_focus = state.is_focus(super::focus::ElementFocus::MethodUrlBar);

        let border_style = is_focus
            .then_some(Style::default().blue())
            .unwrap_or_default();

        let line_block = Block::bordered().border_style(border_style);

        let area = line_block.inner(self.render_area);
        frame.render_widget(line_block, self.render_area);

        let [method_area, left_separator_area, url_area, right_reparator_area, indicator_area] =
            Layout::horizontal([
                Constraint::Length(11),
                Constraint::Length(1),
                Constraint::Min(10),
                Constraint::Length(1),
                Constraint::Length(10),
            ])
            .areas(area);

        frame.render_widget(
            Separator::default().style(border_style),
            left_separator_area,
        );
        frame.render_widget(
            Separator::default().style(border_style),
            right_reparator_area,
        );

        frame.render_widget(
            Span::from(expand(request.method().as_ref(), " ", 10))
                .style(Style::new().on_light_red())
                .black(),
            method_area,
        );

        frame.render_widget(&self.url_input, url_area);
        frame.render_widget(
            Span::from(expand(if self.is_sending { "--" } else { "Send" }, " ", 10))
                .on_light_green()
                .black(),
            indicator_area,
        );
    }

    fn set_area(&mut self, area: Rect) {
        self.render_area = area;
    }

    fn on_key(&mut self, key: crossterm::event::KeyEvent, state: &mut Self::State) {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Tab => {
                    state.set_focus(super::focus::ElementFocus::RequestBuilder);
                    let url = self.url_input.lines()[0].clone();
                    state.change_url_from_state(url);
                }
                KeyCode::BackTab => {
                    state.set_focus(super::focus::ElementFocus::Collections);
                    let url = self.url_input.lines()[0].clone();
                    state.change_url_from_state(url);
                }
                KeyCode::Enter => {
                    // Change to pending
                    if key.modifiers == KeyModifiers::ALT {
                        // Open the method picker
                        state.set_overlay(super::focus::OverlayFocus::MethodSelector);
                        let method = state.current_request().unwrap().method();

                        let mut dropdown_mut = self.dropdown_data_ref.borrow_mut();

                        dropdown_mut.set_area(Rect {
                            x: self.render_area.left(),
                            y: self.render_area.bottom(),
                            width: 11, // length of the largest httpMethod
                            height: 8, // options + 1 (border)
                        });
                        dropdown_mut.select(method);
                    } else {
                        self.is_sending = !self.is_sending;
                    }
                }

                _ => {
                    let key_input = Input::from(key);
                    self.url_input.input(key_input);
                }
            }
        }
    }

    fn on_change_state(&mut self, state: &Self::State) {
        match state.current_request_idx {
            Some(idx) => {
                if self.request_idx != idx {
                    let new_url = state.current_request().unwrap().url();
                    self.clean_url();
                    self.url_input.insert_str(new_url);
                    self.request_idx = idx;
                }
            }
            None => {}
        };
    }
}
