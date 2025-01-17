// use crate::tools::tui::core::elements::Element;
mod state;

use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Layout},
    style::{Style, Stylize},
    text::Span,
    widgets::{Block, Borders, Clear, List, ListState},
};

pub use state::MethodSelectorState;
use state::UrlMethod;

use crate::tools::tui::element::{EffectCommand, Element};

pub struct MethodSelector {
    show_popup: bool,
    methods: ListState,
}

impl MethodSelector {
    pub fn new() -> Self {
        let list_state = ListState::default().with_selected(Some(0));

        Self {
            show_popup: false,
            methods: list_state,
        }
    }

    pub fn methods() -> [String; 4] {
        [
            UrlMethod::Get.to_string(),
            UrlMethod::Post.to_string(),
            UrlMethod::Put.to_string(),
            UrlMethod::Delete.to_string(),
        ]
    }
}

impl Element for MethodSelector {
    type State = MethodSelectorState;

    fn render(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
        state: &Self::State,
    ) {
        let method_block = Block::new().borders(Borders::ALL).title("Method");

        let span = Span::from(state.method().to_string());

        frame.render_widget(span, method_block.inner(area));
        frame.render_widget(method_block, area);

        if self.show_popup {
            let popup_area = {
                let vertical = Layout::vertical([Constraint::Percentage(60)])
                    .flex(ratatui::layout::Flex::Center);

                let horizontal = Layout::horizontal([Constraint::Percentage(40)])
                    .flex(ratatui::layout::Flex::Center);

                let [area] = vertical.areas(frame.area());
                let [area] = horizontal.areas(area);
                area
            };

            let list = List::new([
                UrlMethod::Get.to_string(),
                UrlMethod::Post.to_string(),
                UrlMethod::Put.to_string(),
                UrlMethod::Delete.to_string(),
            ])
            .block(
                Block::new()
                    .borders(Borders::ALL)
                    .title("Select the method type"),
            )
            .highlight_symbol("> ")
            .highlight_style(Style::default().yellow());

            frame.render_widget(Clear, popup_area);
            frame.render_stateful_widget(list, popup_area, &mut self.methods);
        }
    }

    fn event(
        &mut self,
        key: &crossterm::event::KeyEvent,
        state: &mut Self::State,
    ) -> EffectCommand {
        if let KeyCode::Enter = key.code {
            self.show_popup = !self.show_popup;

            if self.show_popup {
                let find = Self::methods()
                    .iter()
                    .enumerate()
                    .find(|(_i, mt)| **mt == state.method().to_string())
                    .map(|(i, _)| i);

                self.methods.select(find);
            } else {
                let find = Self::methods()
                    .get(self.methods.selected().unwrap_or(0))
                    .cloned();

                if let Some(method) = find {
                    state.set_method(method.as_str().into());
                }
            }
        }

        if self.show_popup {
            match key.code {
                KeyCode::Down => {
                    self.methods.select_next();
                }
                KeyCode::Up => {
                    self.methods.select_previous();
                }
                _ => {}
            }
        }

        EffectCommand::Nothing
    }
}
