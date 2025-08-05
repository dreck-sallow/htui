use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout, Margin, Rect},
    style::{Style, Stylize},
    widgets::{Block, Borders, Tabs},
};

use crate::{
    programs::tui::common::component::{Drawable, Interactive, WithHistory},
    store::models::{BodyContent, KeyValueParam},
};

use super::{
    action::PaneAction, body_editor::BodyEditorComponent, params_table::TableParams, ElementFocus,
};

#[derive(Clone, Copy)]
pub enum Tab {
    Headers,
    Body,
    Params,
}

impl Tab {
    pub fn as_idx(&self) -> usize {
        match self {
            Tab::Params => 0,
            Tab::Headers => 1,
            Tab::Body => 2,
        }
    }
}

impl AsRef<str> for Tab {
    fn as_ref(&self) -> &str {
        match self {
            Tab::Headers => "Headers",
            Tab::Body => "Body",
            Tab::Params => "Params",
        }
    }
}

pub struct RequestEditorComponent {
    tab: Tab,
    header_area: Rect,
    render_area: Rect,
    params_table: TableParams,
    headers_table: TableParams,
    // headers_editor: TextEditor,
    body_editor_component: BodyEditorComponent,
}

impl RequestEditorComponent {
    pub fn new() -> Self {
        Self {
            tab: Tab::Params,
            header_area: Rect::default(),
            render_area: Rect::default(),
            params_table: TableParams::new(),
            headers_table: TableParams::new(),
            // headers_editor: TextEditor::new(true),
            body_editor_component: BodyEditorComponent::new(),
        }
    }

    pub fn set_state(
        &mut self,
        params: Vec<KeyValueParam>,
        headers: Vec<KeyValueParam>,
        body: BodyContent,
    ) {
        self.params_table.set_state(params);
        self.headers_table.set_state(headers);
        self.body_editor_component.set_state(body);
    }

    pub fn get_data(&mut self) -> (Vec<KeyValueParam>, Vec<KeyValueParam>, BodyContent) {
        (
            self.params_table.get_data(),
            self.headers_table.get_data(),
            self.body_editor_component.get_data(),
        )
    }

    /// on corners tab, return bool for indicate set focus
    pub fn change_tab(&mut self, is_next: bool) -> bool {
        match self.tab {
            Tab::Headers => {
                if !self.headers_table.is_editing() {
                    if is_next {
                        self.tab = Tab::Body;
                    } else {
                        self.tab = Tab::Params;
                    }
                }
            }
            Tab::Body => {
                if !self.body_editor_component.is_editing() {
                    if is_next {
                        // outof
                        // self.tab = Tab::Params;
                        return true;
                    } else {
                        self.tab = Tab::Headers;
                    }
                }
            }
            Tab::Params => {
                if !self.params_table.is_editing() {
                    if is_next {
                        self.tab = Tab::Headers;
                    } else {
                        return true;
                        // self.tab = Tab::Body;
                    }
                }
            }
        }

        false
    }
}

impl Drawable for RequestEditorComponent {
    type Params = ElementFocus;

    fn draw<'a: 'painter, 'painter>(
        &'a self,
        painter: &mut crate::programs::tui::common::component::Painter<'painter>,
        params: Self::Params,
    ) {
        painter.render(move |frame| {
            let style = if params == ElementFocus::RequestBuilder {
                Style::default().blue()
            } else {
                Style::default()
            };

            let block = Block::bordered().border_style(style);
            frame.render_widget(block, self.render_area);

            let tabs = Tabs::new([
                format!(" {} ", Tab::Params.as_ref()),
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
                self.headers_table.draw(painter, ());
            }
            Tab::Body => {
                self.body_editor_component.draw(painter, ());
            }
            Tab::Params => {
                self.params_table.draw(painter, ());
            }
        }
    }

    fn set_area(&mut self, area: Rect) {
        let main_areas = Layout::vertical([Constraint::Length(2), Constraint::Fill(50)])
            .split(area.inner(Margin::new(1, 1)));

        self.render_area = area;
        self.header_area = main_areas[0];
        self.params_table.set_area(main_areas[1]);
        self.headers_table.set_area(main_areas[1]);
        self.body_editor_component.set_area(main_areas[1]);
    }
}

impl Interactive for RequestEditorComponent {
    type Effect = PaneAction;

    fn on_key(&mut self, key: crossterm::event::KeyEvent) -> Option<Self::Effect> {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Tab => {
                    if self.change_tab(true) {
                        // next focus
                        // return Some(RequestEditorEffect::NextFocus);
                        return Some(PaneAction::NextFocus);
                    }
                }
                KeyCode::BackTab => {
                    if self.change_tab(false) {
                        // previous focus
                        // return Some(RequestEditorEffect::PreviousFocus);
                        return Some(PaneAction::PreviousFocus);
                    }
                }
                _ => match self.tab {
                    Tab::Headers => {
                        self.headers_table.on_key(key);
                    }
                    Tab::Body => {
                        self.body_editor_component.on_key(key);
                    }
                    Tab::Params => {
                        self.params_table.on_key(key);
                    }
                },
            }
        }
        None
    }
}

impl WithHistory for RequestEditorComponent {
    fn undo(&mut self) {
        match self.tab {
            Tab::Headers => self.headers_table.undo(),
            Tab::Body => self.body_editor_component.undo(),
            Tab::Params => self.params_table.undo(),
        }
    }

    fn redo(&mut self) {
        match self.tab {
            Tab::Headers => self.headers_table.redo(),
            Tab::Body => self.body_editor_component.redo(),
            Tab::Params => self.params_table.redo(),
        }
    }
}
