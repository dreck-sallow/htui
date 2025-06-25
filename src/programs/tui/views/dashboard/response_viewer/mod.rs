use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, Borders, Padding, Tabs, Widget},
    Frame,
};
use response_viewer_state::ViewerTabs;

use super::{editor::TextEditor, global_pane_state::GlobalPaneState};

pub mod response_viewer_state;

pub struct ResponseViewerView {
    headers_editor: TextEditor,
}

impl ResponseViewerView {
    pub fn new() -> Self {
        Self {
            headers_editor: TextEditor::new(false),
        }
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect, state: &GlobalPaneState) {
        let response_state = state.response_state_ref();
        let is_focus = state.is_focus(super::focus::ElementFocus::ResponseViewer);

        let block = Block::bordered()
            .title(" Request builder ")
            .border_style(
                is_focus
                    .then_some(Style::default().blue())
                    .unwrap_or_default(),
            )
            .padding(Padding::symmetric(1, 0));

        let [tabs_area, content_area] =
            Layout::vertical([Constraint::Length(2), Constraint::Fill(1)]).areas(block.inner(area));

        frame.render_widget(block, area);

        let tabs = Tabs::new([
            format!(" {} ", Into::<&str>::into(ViewerTabs::Headers)),
            format!(" {} ", Into::<&str>::into(ViewerTabs::Body)),
        ])
        .select(Some(response_state.tab as usize))
        // .highlight_style(
        //     (response_state.focus() == &Focus::Tabs)
        //         .then_some(Style::default().on_light_magenta().black())
        //         .unwrap_or(Style::default().on_dark_gray()),
        // )
        .block(Block::new().borders(Borders::BOTTOM));

        frame.render_widget(tabs, tabs_area);

        match response_state.tab {
            ViewerTabs::Headers => {
                frame.render_widget(&self.headers_editor, content_area);
            }
            ViewerTabs::Body => {}
        }
    }

    pub fn handle_key(&self, key: KeyEvent, state: &mut GlobalPaneState) {}
}
