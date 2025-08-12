use std::rc::Rc;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::Span,
    widgets::{Block, BorderType},
};
use tui_textarea::{CursorMove, Input, TextArea};

use crate::{
    programs::tui::{
        common::{
            action_history::{ActionHistory, History, TrackAction},
            component::{Drawable, Interactive, WithHistory},
        },
        config::Config,
        elements::{dropdown::OverlayDropdown, utils::expand, Separator},
    },
    store::models::HttpMethod,
};

use super::{action::PaneAction, ElementFocus};

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
    config: Rc<Config>,
    _history: ActionHistory<MethodUrlAction>,
}

impl MethodUrlBarComponent {
    pub fn new(config: Rc<Config>) -> Self {
        let mut url_input = TextArea::default();
        url_input.set_cursor_line_style(Style::default());
        url_input.set_placeholder_text("https://");

        Self {
            render_area: Rect::default(),
            method: HttpMethod::Get,
            url_input,
            dropdown: OverlayDropdown::with_items(HttpMethod::Get, METHODS)
                .with_highlight_style(
                    Style::default()
                        .fg(config.theme.dropdown_highlight.fg)
                        .bg(config.theme.dropdown_highlight.bg),
                )
                .with_style(
                    Style::default()
                        .fg(config.theme.dropdown.fg)
                        .bg(config.theme.dropdown.bg),
                ),
            config,
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

    pub fn get_data(&self) -> (HttpMethod, String) {
        (self.method, self.url_input.lines()[0].to_string())
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

            let border_style = Style::default().fg(is_focus
                .then_some(self.config.theme.border_focus)
                .unwrap_or(self.config.theme.border));

            let line_block = Block::bordered()
                .border_type(if is_focus {
                    BorderType::Thick
                } else {
                    BorderType::Plain
                })
                .border_style(border_style);

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
                    .fg(self.config.theme.dropdown.fg)
                    .bg(self.config.theme.dropdown.bg),
                method_area,
            );

            frame.render_widget(&self.url_input, url_area);
            // frame.render_widget(
            //     Span::from(expand(
            //         if self.show_dropdown { "--" } else { "Send" },
            //         " ",
            //         10,
            //     ))
            //     .on_light_green()
            //     .black(),
            //     indicator_area,
            // );
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
    // type Effect = MethodUrlEffect;
    type Effect = PaneAction;

    fn on_key(&mut self, key: KeyEvent) -> Option<Self::Effect> {
        if self.show_dropdown {
            if let Some(action) = self.config.keymap.match_global_action(key) {
                match action {
                    crate::programs::tui::config::keybinding::GlobalKeyAction::ClosePopup => {
                        self.show_dropdown = false;
                    }
                    crate::programs::tui::config::keybinding::GlobalKeyAction::SubmitPopup => {
                        self.show_dropdown = false;
                        self._history.apply(
                            MethodUrlAction::ChangeMethod(self.dropdown.selected().to_owned()),
                            &mut self.method,
                        );
                    }
                    crate::programs::tui::config::keybinding::GlobalKeyAction::MoveDown => {
                        self.dropdown.next();
                        
                    },
                    crate::programs::tui::config::keybinding::GlobalKeyAction::MoveUp => {
                        self.dropdown.prev();
                        
                    }
                    _ => {},
                }                
            }
        } else {
            let is_key_consumed = match self.config.keymap.match_global_action(key) {
                Some(action) => match action {
                    crate::programs::tui::config::keybinding::GlobalKeyAction::NextFocus => {
                        return Some(PaneAction::NextFocus);
                    }
                    crate::programs::tui::config::keybinding::GlobalKeyAction::PreviousFocus => {
                        return Some(PaneAction::PreviousFocus);
                    }
                    _ => false,
                },
                None => false,
            };

            if !is_key_consumed {
                match self.config.keymap.match_method_url_action(key) {
                    Some(action) => match action {
                        crate::programs::tui::config::keybinding::MethodUrlKeyAction::OpenDropdown => {
                            self.show_dropdown = true;                            
                        },
                    },
                    None => {
                        // Avoid new line on enter
                        if key.code != KeyCode::Enter {
                            self.url_input.input(Input::from(key));
                        }
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
                *state = *http_method;
                Some(MethodUrlAction::ChangeMethod(previous_method))
            }
        }
    }
}
