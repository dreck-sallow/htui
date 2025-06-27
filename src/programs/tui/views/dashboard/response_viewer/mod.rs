use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, Borders, Padding, Tabs},
    Frame,
};
use response_viewer_state::ViewerTabs;

use crate::programs::tui::element_view::ElementView;

use super::{editor::TextEditor, global_pane_state::GlobalPaneState};

pub mod response_viewer_state;

pub struct ResponseViewerView {
    render_area: Rect,
    headers_editor: TextEditor,
}

impl ResponseViewerView {
    pub fn new() -> Self {
        Self {
            render_area: Rect::default(),
            headers_editor: TextEditor::new(false),
        }
    }
}

impl ElementView for ResponseViewerView {
    type State = GlobalPaneState;

    fn draw(&self, frame: &mut Frame, state: &Self::State) {
        let response_state = state.response_state_ref();
        let is_focus = state.is_focus(super::focus::ElementFocus::ResponseViewer);

        let block = Block::bordered()
            .title(" Response viewer ")
            .border_style(
                is_focus
                    .then_some(Style::default().blue())
                    .unwrap_or_default(),
            )
            .padding(Padding::symmetric(1, 0));

        let [tabs_area, content_area] =
            Layout::vertical([Constraint::Length(2), Constraint::Fill(1)])
                .areas(block.inner(self.render_area));

        frame.render_widget(block, self.render_area);

        let tabs = Tabs::new([
            format!(" {} ", Into::<&str>::into(ViewerTabs::Headers)),
            format!(" {} ", Into::<&str>::into(ViewerTabs::Body)),
        ])
        .select(Some(response_state.tab as usize))
        .highlight_style(
            (state.response_state_ref().is_tab_focus)
                .then_some(Style::default().on_light_magenta().black())
                .unwrap_or(Style::default().on_dark_gray()),
        )
        .block(Block::new().borders(Borders::BOTTOM));

        frame.render_widget(tabs, tabs_area);

        match response_state.tab {
            ViewerTabs::Headers => {
                frame.render_widget(&self.headers_editor, content_area);
            }
            ViewerTabs::Body => {}
        }
    }

    fn set_area(&mut self, area: Rect) {
        self.render_area = area;
    }

    fn on_key(&mut self, key: KeyEvent, state: &mut Self::State) {
        if key.kind == KeyEventKind::Press {
            let is_tab_focus = state.response_state_ref().is_tab_focus;

            if is_tab_focus {
                match key.code {
                    KeyCode::Left | KeyCode::Char('h') => {
                        match state.response_state_ref().tab {
                            ViewerTabs::Headers => {}
                            ViewerTabs::Body => {
                                state.response_state_mut().tab = ViewerTabs::Headers;
                            }
                        };
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        match state.response_state_ref().tab {
                            ViewerTabs::Headers => {
                                state.response_state_mut().tab = ViewerTabs::Body;
                            }
                            ViewerTabs::Body => {}
                        };
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        state.response_state_mut().is_tab_focus = false;
                    }
                    KeyCode::Tab => {
                        state.response_state_mut().is_tab_focus = false;
                    }
                    KeyCode::BackTab => {
                        state.set_focus(super::focus::ElementFocus::RequestBuilder);
                    }
                    _ => {}
                }
            } else {
                match key.code {
                    KeyCode::Tab => {
                        state.set_focus(super::focus::ElementFocus::Collections);
                    }
                    KeyCode::BackTab => {
                        state.response_state_mut().is_tab_focus = true;
                    }
                    _ => match state.response_state_ref().tab {
                        ViewerTabs::Headers => self.headers_editor.handle_key(key),
                        ViewerTabs::Body => {}
                    },
                }
            }
        }
    }

    fn on_change_state(&mut self, _state: &Self::State) {}
}
