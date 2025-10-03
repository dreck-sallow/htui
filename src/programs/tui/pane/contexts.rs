use std::ops::Not;

use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Rect},
    style::{Style, Stylize},
    widgets::Block,
};

use crate::{
    app_project::models::ContextEnv,
    programs::tui::{
        common::{
            input::mode_input::ModeInput, list_utils, Interactive, UiComposedElement, UiElementV2,
        },
        config::{keybinding::GlobalKeyAction, Config},
        elements::utils::{center_area, expand},
    },
};

use super::{action::PaneAction, ElementFocus};

enum OverlayType {
    EditContext,
    NewContext,
    None,
}

pub struct EnvironmentContexts {
    render_area: Rect,
    list: Vec<ContextEnv>,
    /// Index for current visual context
    current: Option<usize>,
    input: ModeInput,
    overlay_type: OverlayType,
}

impl EnvironmentContexts {
    pub fn from_list(list: Vec<ContextEnv>) -> Self {
        let current = list.is_empty().not().then_some(0);
        Self {
            list,
            current,
            input: ModeInput::new(""),
            render_area: Rect::default(),
            overlay_type: OverlayType::None,
        }
    }
}

impl<'params> UiComposedElement<'params> for EnvironmentContexts {
    type Params = (bool, &'params Config);

    fn set_area(&mut self, area: ratatui::prelude::Rect, _viewport_area: ratatui::prelude::Rect) {
        self.render_area = area;
        self.input.set_visual_width(area.width.saturating_sub(2));
    }

    fn draw(&self, (is_focus, config): Self::Params, frame: &mut ratatui::Frame) {
        let block = Block::bordered().border_style(Style::default().fg(if is_focus {
            config.theme.border_focus
        } else {
            config.theme.border
        }));

        let area = block.inner(self.render_area);

        frame.render_widget(block, self.render_area);

        match self.current {
            Some(idx) => {
                let context = &self.list[idx];
                frame.render_widget(
                    expand(context.name.as_str(), " ", area.width as usize).blue(),
                    area,
                );
            }
            None => {
                frame.render_widget(
                    expand("No context selected", " ", area.width as usize)
                        .italic()
                        .dark_gray(),
                    area,
                );
            }
        }
    }

    fn draw_overlay(&self, (_focus, config): Self::Params, frame: &mut ratatui::Frame) {
        let title = match self.overlay_type {
            OverlayType::EditContext => " Edit name ",
            OverlayType::NewContext => "Context name ",
            OverlayType::None => {
                return;
            }
        };

        let area = center_area(
            frame.area(),
            Constraint::Length(3),
            Constraint::Percentage(40),
        );

        let block = Block::bordered()
            .title(title)
            .border_style(Style::default().fg(config.theme.border_focus));

        let inner_area = block.inner(area);

        frame.render_widget(block, area);

        self.input.draw(inner_area, frame);
    }
}

impl<'params> Interactive<'params> for EnvironmentContexts {
    type Effect = PaneAction;

    type Params = (ElementFocus, &'params Config);

    fn handle_key(
        &mut self,
        (_focus, config): Self::Params,
        key: crossterm::event::KeyEvent,
    ) -> Self::Effect {
        match self.overlay_type {
            OverlayType::EditContext => todo!(),
            OverlayType::NewContext => {
                let mut consumed = false;
                if let Some(action) = config.keymap.match_global_action(key) {
                    match action {
                        GlobalKeyAction::ClosePopup => {
                            self.overlay_type = OverlayType::None;
                            consumed = true;
                        }
                        GlobalKeyAction::SubmitPopup => {
                            if self.input.txt().chars().next().is_some() {
                                self.overlay_type = OverlayType::None;
                                self.list.push(ContextEnv::new(self.input.txt()));
                                self.current = Some(self.list.len() - 1);
                                self.input.clear();
                            }
                            consumed = true;
                        }
                        _ => {}
                    }
                }

                if !consumed {
                    self.input.handle_key(key);
                }
            }
            OverlayType::None => {
                let mut consumed = false;
                if let Some(action) = config.keymap.match_global_action(key) {
                    match action {
                        GlobalKeyAction::MoveLeft => {
                            self.current = list_utils::prev(self.current);
                            consumed = true;
                        }
                        GlobalKeyAction::MoveRight => {
                            self.current = list_utils::next(self.current, self.list.len());
                            consumed = true;
                        }
                        _ => {}
                    }
                }

                if !consumed {
                    if let KeyCode::Char(ch) = key.code {
                        match ch {
                            'e' => {
                                self.overlay_type = OverlayType::EditContext;
                                let context = &self.list[self.current.unwrap()];
                                self.input.replace(&context.name);
                            }
                            'd' => {
                                // I should delete the current context
                                self.current =
                                    list_utils::delete_element(self.current, &mut self.list);
                            }
                            'n' => {
                                self.overlay_type = OverlayType::NewContext;
                            }
                            _ => {}
                        }
                    }

                    if let KeyCode::Esc = key.code {
                        return PaneAction::RestoreFocus;
                    }
                }
            }
        }

        PaneAction::Noop
    }
}
