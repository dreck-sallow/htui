use std::{collections::HashMap, path::PathBuf};

use crossterm::event::{KeyCode, KeyEventKind};

use crate::{
    programs::tui::{
        element_view::ElementView,
        elements::dropdown::{OverlayDropdown, SharedDropdown},
    },
    store::models::BodyContent,
};

use super::global_pane_state::GlobalPaneState;

/// Body content type
/// mirror from `BodyContent`
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BodyType {
    Empty,
    File,
    Form,
    Text,
}

impl AsRef<str> for BodyType {
    fn as_ref(&self) -> &str {
        match self {
            BodyType::Empty => "Empty",
            BodyType::File => "File",
            BodyType::Form => "Form",
            BodyType::Text => "Text",
        }
    }
}

impl From<&BodyContent> for BodyType {
    fn from(value: &BodyContent) -> Self {
        match value {
            BodyContent::Empty => Self::Empty,
            BodyContent::File(_) => Self::File,
            BodyContent::Form(_) => Self::Form,
            BodyContent::Text(_) => Self::Text,
        }
    }
}

impl From<&mut BodyContent> for BodyType {
    fn from(value: &mut BodyContent) -> Self {
        match value {
            BodyContent::Empty => Self::Empty,
            BodyContent::File(_) => Self::File,
            BodyContent::Form(_) => Self::Form,
            BodyContent::Text(_) => Self::Text,
        }
    }
}

impl Into<BodyContent> for BodyType {
    fn into(self) -> BodyContent {
        match self {
            BodyType::Empty => BodyContent::Empty,
            BodyType::File => BodyContent::File(PathBuf::new()),
            BodyType::Form => BodyContent::Form(HashMap::new()),
            BodyType::Text => BodyContent::Text("".into()),
        }
    }
}

pub struct BodyTypeSelectorView {
    inner: OverlayDropdown<BodyType>,
}

impl BodyTypeSelectorView {
    pub fn new(state: SharedDropdown<BodyType>) -> Self {
        Self {
            inner: OverlayDropdown::with_items(
                state,
                [
                    BodyType::Empty,
                    BodyType::Text,
                    BodyType::Form,
                    BodyType::File,
                ],
            ),
        }
    }
}

impl ElementView for BodyTypeSelectorView {
    type State = GlobalPaneState;

    fn draw(&self, frame: &mut ratatui::Frame, _state: &Self::State) {
        self.inner.draw(frame);
    }

    fn on_key(&mut self, key: crossterm::event::KeyEvent, state: &mut Self::State) {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Char('j') | KeyCode::Down => {
                    self.inner.next();
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.inner.prev();
                }
                KeyCode::Enter => {
                    state.hidden_overlay();
                    // state.set_current_request_method(self.inner.selected());
                }
                KeyCode::Esc => {
                    state.hidden_overlay();
                }
                _ => {}
            }
        }
    }
}
