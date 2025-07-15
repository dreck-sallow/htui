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
        common::{
            action_history::{ActionHistory, History, TrackAction},
            component::{Drawable, Interactive, WithHistory},
        },
        elements::{dropdown::OverlayDropdown, utils::expand, Separator},
    },
    store::models::HttpMethod,
};

use super::state::ElementFocus;

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
    dropdown: OverlayDropdown<HttpMethod>,
    show_dropdown: bool,
    _history: ActionHistory<MethodUrlAction>,
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
            dropdown: OverlayDropdown::with_items(HttpMethod::Get, METHODS)
                .with_highlight_style(Style::default().on_light_blue())
                .with_style(Style::default().on_light_red()),
            show_dropdown: false,
            _history: ActionHistory::new(),
        }
    }

    fn clean_url(&mut self) {
        self.url_input.move_cursor(CursorMove::End);
        self.url_input.delete_line_by_head();
    }

    pub fn set_data(&mut self, method: HttpMethod, url: &str) {
        self.method = method;
        self.clean_url();
        self.url_input.insert_str(url);

        self._history.clean();
    }
}

impl Drawable for MethodUrlBarComponent {
    type Params = ElementFocus;

    fn set_area(&mut self, _area: Rect) {
        self.render_area = _area;
    }

    fn draw<'a: 'painter, 'painter>(
        &'a self,
        painter: &mut crate::programs::tui::common::component::Painter<'painter>,
        params: Self::Params,
    ) {
        painter.render(move |frame| {
            let is_focus = params == ElementFocus::MethodUrlBar;

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
                    x: self.render_area.left() + 1,
                    y: self.render_area.bottom() - 1,
                    width: 10,
                    height: METHODS.len() as u16,
                };
                frame.render_widget(&self.dropdown, area);
            });
        }
    }
}

impl Interactive for MethodUrlBarComponent {
    type Effect = MethodUrlEffect;

    fn on_key(&mut self, key: KeyEvent) -> Option<Self::Effect> {
        if key.kind == KeyEventKind::Press {
            if self.show_dropdown {
                match key.code {
                    KeyCode::Enter => {
                        self.show_dropdown = false;
                        self._history.apply(
                            MethodUrlAction::ChangeMethod(self.dropdown.selected().to_owned()),
                            &mut self.method,
                        );
                    }
                    KeyCode::Esc => {
                        self.show_dropdown = false;
                    }
                    _ => {
                        self.dropdown.handle_key(key);
                    }
                }
            } else {
                match key.code {
                    KeyCode::Tab => {
                        // mutate_on_blur(true);

                        return Some(MethodUrlEffect::NextFocus);
                    }
                    KeyCode::BackTab => {
                        // mutate_on_blur(false);
                        return Some(MethodUrlEffect::PreviousFocus);
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

        None
    }
}

impl WithHistory for MethodUrlBarComponent {
    fn undo(&mut self) {
        self._history.undo(&mut self.method);
    }

    fn redo(&mut self) {
        self._history.redo(&mut self.method);
    }
}

enum MethodUrlAction {
    ChangeMethod(HttpMethod),
}

impl TrackAction for MethodUrlAction {
    type State = HttpMethod;

    fn apply(&self, state: &mut Self::State) -> Option<Self>
    where
        Self: Sized,
    {
        match self {
            MethodUrlAction::ChangeMethod(http_method) => {
                let previous_method = state.clone();
                *state = http_method.clone();
                Some(MethodUrlAction::ChangeMethod(previous_method))
            }
        }
    }
}

pub enum MethodUrlEffect {
    NextFocus,
    PreviousFocus,
}
