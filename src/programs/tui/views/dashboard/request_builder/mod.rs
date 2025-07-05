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

pub mod request_builder_state;

#[derive(Clone, Copy)]
pub enum Tab {
    Headers,
    Body,
}

impl Tab {
    pub fn as_idx(&self) -> usize {
        match self {
            Tab::Headers => 0,
            Tab::Body => 1,
        }
    }
}

impl AsRef<str> for Tab {
    fn as_ref(&self) -> &str {
        match self {
            Tab::Headers => "Headers",
            Tab::Body => "Body",
        }
    }
}

pub struct RequestEditorComponent {
    render_area: Rect,
    header_area: Rect,
    content_area: Rect,
    tab: Tab,
    headers_editor: TextEditor,
    body_editor: BodyEditorComponent,
}

impl RequestEditorComponent {
    pub fn new() -> Self {
        Self {
            render_area: Rect::default(),
            header_area: Rect::default(),
            content_area: Rect::default(),
            tab: Tab::Headers,
            headers_editor: TextEditor::new(true),
            body_editor: BodyEditorComponent::new(true),
        }
    }
}

impl<'a: 'painter, 'painter> Drawable<'a, 'painter> for RequestEditorComponent {
    type State = PaneState;

    fn draw(
        &'a self,
        painter: &mut crate::programs::tui::element_view::Painter<'painter>,
        state: &'a Self::State,
    ) {
        painter.render(|frame| {
            let style = if state.is_focused(super::focus::ElementFocus::RequestBuilder) {
                Style::default().blue()
            } else {
                Style::default()
            };

            let block = Block::bordered().border_style(style);
            frame.render_widget(block, self.render_area);

            let tabs = Tabs::new([
                format!(" {} ", Tab::Headers.as_ref()),
                format!(" {} ", Tab::Body.as_ref()),
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
            Tab::Body => {
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

impl<'a: 'painter, 'painter> Interactive<'a, 'painter> for RequestEditorComponent {
    type Mutator = MutationCollector<'a>;

    fn on_key(
        &mut self,
        key: crossterm::event::KeyEvent,
        mutator: &mut Self::Mutator,
        state: &Self::State,
    ) {
        if key.kind == KeyEventKind::Press {
            let is_editing = match self.tab {
                Tab::Headers => self.headers_editor.mode().is_write_mode(),
                Tab::Body => self.body_editor.is_editing(),
            };

            match key.code {
                KeyCode::Tab => match self.tab {
                    Tab::Headers => {
                        if is_editing {
                            self.headers_editor.handle_key(key)
                        } else {
                            self.tab = Tab::Body;
                        }
                    }
                    Tab::Body => {
                        if is_editing {
                            self.body_editor.on_key(key, mutator, state);
                        } else {
                            mutator.add(SetFocus::for_next());
                        }
                    }
                },
                KeyCode::BackTab => match self.tab {
                    Tab::Headers => {
                        if is_editing {
                            self.headers_editor.handle_key(key);
                        } else {
                            mutator.add(SetFocus::for_previous());
                        }
                    }
                    Tab::Body => {
                        if is_editing {
                            self.body_editor.on_key(key, mutator, state);
                        } else {
                            self.tab = Tab::Headers;
                        }
                    }
                },
                _ => match self.tab {
                    Tab::Headers => self.headers_editor.handle_key(key),
                    Tab::Body => {
                        self.body_editor.on_key(key, mutator, state);
                    }
                },
            }
        }
    }

    fn on_change_state(&mut self, state: &Self::State) {
        if let Some(req) = state.reader().current_request() {
            self.headers_editor.clean_lines();
            for (k, v) in req.headers_map() {
                self.headers_editor.insert_str(&format!("{k}:{v}"));
            }
        }
        self.body_editor.on_change_state(state);
    }
}
