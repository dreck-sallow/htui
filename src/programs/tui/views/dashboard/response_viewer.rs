use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout, Margin, Rect},
    style::{Style, Stylize},
    widgets::{Block, Borders, Tabs},
};

use crate::programs::tui::element_view::{Drawable, Interactive};

use super::{
    body_editor::BodyEditorComponent,
    editor::TextEditor,
    pane_state::{history::MutationCollector, mutations::SetFocus, PaneState},
};

#[derive(Clone, Copy)]
pub enum Tab {
    Headers,
    Response,
}

impl Tab {
    pub fn as_idx(&self) -> usize {
        match self {
            Tab::Headers => 1,
            Tab::Response => 0,
        }
    }
}

impl AsRef<str> for Tab {
    fn as_ref(&self) -> &str {
        match self {
            Tab::Headers => "Headers",
            Tab::Response => "Response",
        }
    }
}

pub struct ResponseViewerComponent {
    render_area: Rect,
    header_area: Rect,
    content_area: Rect,
    tab: Tab,
    headers_editor: TextEditor,
    body_editor: BodyEditorComponent,
}

impl ResponseViewerComponent {
    pub fn new() -> Self {
        Self {
            render_area: Rect::default(),
            header_area: Rect::default(),
            content_area: Rect::default(),
            tab: Tab::Response,
            headers_editor: TextEditor::new(false),
            body_editor: BodyEditorComponent::new(false),
        }
    }
}

impl<'a: 'painter, 'painter> Drawable<'a, 'painter> for ResponseViewerComponent {
    type State = PaneState;

    fn draw(
        &'a self,
        painter: &mut crate::programs::tui::element_view::Painter<'painter>,
        state: &'a Self::State,
    ) {
        painter.render(|frame| {
            let style = if state.is_focused(super::focus::ElementFocus::ResponseViewer) {
                Style::default().blue()
            } else {
                Style::default()
            };

            let block = Block::bordered().border_style(style);
            frame.render_widget(block, self.render_area);

            let tabs = Tabs::new([
                format!(" {} ", Tab::Response.as_ref()),
                format!(" {} ", Tab::Headers.as_ref()),
            ])
            .select(self.tab.as_idx())
            .block(Block::new().borders(Borders::BOTTOM).border_style(style))
            .highlight_style(Style::default().blue());

            frame.render_widget(tabs, self.header_area);
        });

        match self.tab {
            Tab::Headers => {
                painter.render(|frame| {
                    frame.render_widget(&self.headers_editor, self.content_area);
                });
            }
            Tab::Response => {
                self.body_editor.draw(painter, state);
            }
        }
    }

    fn set_render_area(&mut self, area: Rect) {
        let main_areas = Layout::vertical([Constraint::Length(2), Constraint::Fill(50)])
            .split(area.inner(Margin::new(1, 1)));

        self.render_area = area;
        self.header_area = main_areas[0];
        self.content_area = main_areas[1];
        self.body_editor.set_render_area(main_areas[1]);
    }
}

impl<'a: 'painter, 'painter> Interactive<'a, 'painter> for ResponseViewerComponent {
    type Mutator = MutationCollector<'a>;

    fn on_key(
        &mut self,
        key: crossterm::event::KeyEvent,
        mutator: &mut Self::Mutator,
        state: &Self::State,
    ) {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Tab => match self.tab {
                    Tab::Headers => {
                        mutator.add(SetFocus::for_next());
                    }
                    Tab::Response => {
                        self.tab = Tab::Headers;
                    }
                },
                KeyCode::BackTab => match self.tab {
                    Tab::Headers => {
                        self.tab = Tab::Response;
                    }
                    Tab::Response => {
                        mutator.add(SetFocus::for_previous());
                    }
                },
                _ => match self.tab {
                    Tab::Headers => self.headers_editor.handle_key(key),
                    Tab::Response => {
                        self.body_editor.on_key(key, mutator, state);
                    }
                },
            }
        }
    }
}
