use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Layout, Margin, Rect},
    style::{Style, Stylize},
    text::Span,
    widgets::{Block, BorderType},
};

use crate::{
    app_project::models::HttpMethod, programs::tui::{
        common::{
            action_history::{ActionHistory, History, TrackAction}, component::WithHistory, input::mode_input::ModeInput, Interactive, UiComposedElement, UiElementV2
        },
        config::Config,
        elements::{dropdown::OverlayDropdown, utils::expand, Separator},
    }
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
    url_input: ModeInput,
    dropdown: OverlayDropdown<HttpMethod>,
    show_dropdown: bool,
    _history: ActionHistory<MethodUrlAction>,
}

impl MethodUrlBarComponent {
    pub fn new(config: &Config) -> Self {
        Self {
            render_area: Rect::default(),
            method: HttpMethod::Get,
            url_input: ModeInput::new("https://"),
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
            show_dropdown: false,
            _history: ActionHistory::new(),
        }
    }

    pub fn set_data(&mut self, method: HttpMethod, url: &str) {
        self.method = method;
        self.url_input.replace(url);
        self._history.clean();
    }

    pub fn get_data(&self) -> (HttpMethod, String) {
        (self.method, self.url_input.txt().to_string())
    }


    fn areas(area: Rect) -> [Rect;5] {
        Layout::horizontal([
            Constraint::Length(11),
            Constraint::Length(1),
            Constraint::Min(10),
            Constraint::Length(1),
            Constraint::Length(10),
        ])
        .areas(area)
    }
}


impl<'params> UiComposedElement<'params> for MethodUrlBarComponent {
    type Params = (ElementFocus, &'params Config);

    fn set_area(&mut self, area: Rect, _viewport_area: Rect) {
        self.render_area = area;
        let [_,_,url_area,_,_] = Self::areas(area.inner(Margin::new(1, 1)));
        self.url_input.set_visual_width(url_area.width);
    }

    fn draw(&self, (focus, config): Self::Params, frame: &mut ratatui::Frame) {
        let is_focus = focus == ElementFocus::MethodUrlBar;

            let border_style = Style::default().fg(is_focus
                .then_some(config.theme.border_focus)
                .unwrap_or(config.theme.border));

            let line_block = Block::bordered()
                .border_type(if is_focus {
                    BorderType::Thick
                } else {
                    BorderType::Plain
                })
                .border_style(border_style);

            let area = line_block.inner(self.render_area);
            frame.render_widget(line_block, self.render_area);

            let [method_area, left_separator_area, url_area, right_reparator_area, _indicator_area] = Self::areas(area);

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
                    .fg(config.theme.dropdown.fg)
                    .bg(config.theme.dropdown.bg),
                method_area,
            );

            self.url_input.draw(url_area, frame);

            // frame.render_widget(&self.url_input, url_area);
    }

    fn draw_overlay(&self, _params: Self::Params, frame: &mut ratatui::Frame) {
        if self.show_dropdown {
            let area = Rect {
                    x: self.render_area.left() + 1,
                    y: self.render_area.bottom() - 1,
                    width: 10,
                    height: METHODS.len() as u16,
                };
                frame.render_widget(&self.dropdown, area);
        }        
    }
}

impl<'params> Interactive<'params> for MethodUrlBarComponent {
    type Effect = PaneAction;

    type Params = &'params Config;

    fn is_input_focus(&self) -> bool {
        !self.show_dropdown
    }

    fn is_visible_overlay(&self) -> bool {
        self.show_dropdown
    }

    fn handle_key(&mut self, config: Self::Params, key: KeyEvent) -> Self::Effect {
        if self.show_dropdown {
            if let Some(action) = config.keymap.match_global_action(key) {
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
            let is_key_consumed = match config.keymap.match_global_action(key) {
                Some(action) => match action {
                    crate::programs::tui::config::keybinding::GlobalKeyAction::NextFocus if !self.url_input.mode().is_write_mode() => {
                        return PaneAction::NextFocus;
                    }
                    crate::programs::tui::config::keybinding::GlobalKeyAction::PreviousFocus if !self.url_input.mode().is_write_mode() => {
                        return PaneAction::PreviousFocus;
                    }
                    _ => false,
                },
                None => false,
            };

            if !is_key_consumed {
                match config.keymap.match_method_url_action(key) {
                    Some(action) => match action {
                        crate::programs::tui::config::keybinding::MethodUrlKeyAction::OpenDropdown => {
                            self.show_dropdown = true;                            
                        },
                    },
                    None => {
                        // Avoid new line on enter
                        if key.code != KeyCode::Enter {
                            self.url_input.handle_key(key);
                            // self.url_input.input(Input::from(key));
                        }
                    }
                }
            }
        }

        PaneAction::Noop
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
