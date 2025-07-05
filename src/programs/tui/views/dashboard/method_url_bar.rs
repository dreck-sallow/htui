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
        element_view::{Drawable, Interactive},
        elements::{dropdown::OverlayDropdown_v2, utils::expand, Separator},
    },
    store::models::HttpMethod,
};

use super::pane_state::{
    history::MutationCollector,
    mutations::{EditRequest, FocusNavigation, RequestEditType, SetFocus},
    PaneState,
};

const METHODS: [HttpMethod; 7] = [
    HttpMethod::Get,
    HttpMethod::Post,
    HttpMethod::Put,
    HttpMethod::Patch,
    HttpMethod::Delete,
    HttpMethod::Head,
    HttpMethod::Options,
];

pub struct MethodUrlBarComponent {
    render_area: Rect,
    method: HttpMethod,
    url_input: TextArea<'static>,
    dropdown: OverlayDropdown_v2<HttpMethod>,
    show_dropdown: bool,
}

impl MethodUrlBarComponent {
    pub fn new() -> Self {
        let mut url_input = TextArea::default();
        url_input.set_cursor_line_style(Style::default());
        url_input.set_placeholder_text("https://");

        Self {
            render_area: Rect::default(),
            method: HttpMethod::Get,
            url_input,
            dropdown: OverlayDropdown_v2::with_items(HttpMethod::Get, METHODS)
                .with_highlight_style(Style::default().on_light_blue()),
            show_dropdown: false,
        }
    }

    fn clean_url(&mut self) {
        self.url_input.move_cursor(CursorMove::End);
        self.url_input.delete_line_by_head();
    }
}

impl<'a: 'painter, 'painter> Drawable<'a, 'painter> for MethodUrlBarComponent {
    type State = PaneState;

    fn draw(
        &'a self,
        painter: &mut crate::programs::tui::element_view::Painter<'painter>,
        state: &'a Self::State,
    ) {
        painter.render(|frame| {
            let is_focus = state.is_focused(super::focus::ElementFocus::MethodUrlBar);

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
                Span::from(expand(self.method.as_ref(), " ", 10))
                    .style(Style::new().on_light_red())
                    .black(),
                method_area,
            );

            frame.render_widget(&self.url_input, url_area);
            frame.render_widget(
                Span::from(expand(
                    if self.show_dropdown { "--" } else { "Send" },
                    " ",
                    10,
                ))
                .on_light_green()
                .black(),
                indicator_area,
            );
        });

        if self.show_dropdown {
            painter.render_last(|frame| {
                let area = Rect {
                    x: self.render_area.left(),
                    y: self.render_area.bottom(),
                    width: 11,
                    height: METHODS.len() as u16 + 1,
                };
                frame.render_widget(&self.dropdown, area);
            });
        }
    }

    fn set_render_area(&mut self, _area: Rect) {
        self.render_area = _area;
    }
}

impl<'a: 'painter, 'painter> Interactive<'a, 'painter> for MethodUrlBarComponent {
    type Mutator = MutationCollector<'a>;

    fn on_key(&mut self, key: KeyEvent, mutator: &mut Self::Mutator, _state: &Self::State) {
        if key.kind == KeyEventKind::Press {
            if self.show_dropdown {
                match key.code {
                    KeyCode::Enter => {
                        self.method = self.dropdown.selected().clone();
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
                let mut mutate_on_blur = |focus_navigation: FocusNavigation| {
                    let request_idx = _state.reader().current_request_idx().unwrap();
                    mutator.add(EditRequest::new(
                        request_idx,
                        RequestEditType::Url(self.url_input.lines()[0].to_string()),
                    ));

                    mutator.add(EditRequest::new(
                        request_idx,
                        RequestEditType::Method(self.method),
                    ));
                    mutator.add(SetFocus::new(focus_navigation));
                };

                match key.code {
                    KeyCode::Tab => {
                        mutate_on_blur(FocusNavigation::Next);
                    }
                    KeyCode::BackTab => {
                        mutate_on_blur(FocusNavigation::Prev);
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
    }

    fn on_change_state(&mut self, _state: &Self::State) {
        let reader = _state.reader();

        if let Some(req) = reader.current_request() {
            // FIXME: check for previous request, or react only when change request index not on all mutations
            self.clean_url();
            self.url_input.insert_str(req.url());
            self.method = req.method();
        }
    }
}
