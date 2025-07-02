use body_editor::BodyEditor;
use ratatui::{
    layout::{Constraint, Layout, Margin, Rect},
    text::Span,
    widgets::{Block, Borders},
};

use crate::programs::tui::element_view::ElementView;

use super::{editor::TextEditor, global_pane_state::GlobalPaneState};

pub mod body_editor;

pub struct HttpPayloadEditorView {
    render_area: Rect,
    headers_area: Rect,
    headers_editor: TextEditor,
    body_editor: BodyEditor,
    title: &'static str,
}

impl HttpPayloadEditorView {
    pub fn new(editable: bool, title: &'static str) -> Self {
        Self {
            render_area: Rect::default(),
            headers_area: Rect::default(),
            headers_editor: TextEditor::new(editable),
            body_editor: BodyEditor::new(editable),
            title,
        }
    }
}

impl ElementView for HttpPayloadEditorView {
    type State = GlobalPaneState;

    fn draw(&self, frame: &mut ratatui::Frame, state: &Self::State) {
        let block = Block::bordered().title(self.title);

        // Render headers editor
        frame.render_widget(block, self.render_area);

        let headers_block = Block::new().borders(Borders::RIGHT);

        {
            let [title_area, content_area] =
                Layout::vertical([Constraint::Length(1), Constraint::Fill(1)])
                    .areas(headers_block.inner(self.headers_area));

            frame.render_widget(Span::from(" Headers"), title_area);
            frame.render_widget(&self.headers_editor, content_area);
        }

        frame.render_widget(headers_block, self.headers_area);

        self.body_editor
            .draw(frame, state.current_request().unwrap().body());
    }

    fn set_area(&mut self, area: Rect) {
        let main_areas =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(area.inner(Margin::new(1, 1)));
        self.render_area = area;
        self.headers_area = main_areas[0];
        self.body_editor.set_area(main_areas[1]);
    }
}
