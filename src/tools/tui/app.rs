use std::io::Result;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use super::{
    element::Element,
    elements::{
        collections::{CollectionState, Collections},
        method_selector::{MethodSelector, MethodSelectorState},
        url_input::{state::UrlInputState, UrlInput},
        ElementType,
    },
};

#[derive(Default)]
pub struct AppState {
    collections: CollectionState,
    url_input: UrlInputState,
    method_selector: MethodSelectorState,
}

impl AppState {
    pub fn with_collections(mut self, collections: CollectionState) -> Self {
        self.collections = collections;
        self
    }
}

pub struct App<'a> {
    state: AppState,
    element_show: ElementType,
    url_input: UrlInput<'a>,
    method_selector: MethodSelector,
    collections: Collections,
}

impl<'a> App<'a> {
    pub fn new(state: AppState) -> Self {
        let url_input = UrlInput::new();
        let method_selector = MethodSelector::new();

        Self {
            state,
            element_show: ElementType::Collections,
            url_input,
            method_selector,
            collections: Collections::new(),
        }
    }

    pub fn render(&mut self, frame: &mut Frame) -> Result<()> {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
            .split(frame.area());

        self.collections
            .render(frame, chunks[0], &self.state.collections);

        let right_chunks = Layout::new(
            Direction::Vertical,
            [Constraint::Length(3), Constraint::Fill(1)],
        )
        .split(chunks[1]);

        let top_right_chunks = Layout::new(
            ratatui::layout::Direction::Horizontal,
            [Constraint::Length(10), Constraint::Fill(1)],
        )
        .split(right_chunks[0]);

        self.method_selector
            .render(frame, top_right_chunks[0], &self.state.method_selector);

        self.url_input
            .render(frame, top_right_chunks[1], &self.state.url_input);

        Ok(())
    }

    pub fn handle(&mut self, event: &KeyEvent) {
        match event {
            KeyEvent {
                code: KeyCode::Tab,
                kind: KeyEventKind::Press,
                ..
            } => self.element_show = self.element_show.next(),
            KeyEvent {
                code: KeyCode::BackTab,
                kind: KeyEventKind::Press,
                ..
            } => self.element_show = self.element_show.prev(),

            _ => {
                let effect_command = match self.element_show {
                    ElementType::Collections => {
                        self.collections.event(event, &mut self.state.collections)
                    }
                    ElementType::MethodSelector => self
                        .method_selector
                        .event(event, &mut self.state.method_selector),
                    ElementType::UrlInput => self.url_input.event(event, &mut self.state.url_input),
                };

                match effect_command {
                    super::element::EffectCommand::SetRequest { method, url } => {
                        self.state
                            .method_selector
                            .set_method(method.as_str().into());
                        self.state.url_input.set_url(url.clone());
                    }
                    super::element::EffectCommand::Nothing => {}
                }
            }
        }
    }
}
