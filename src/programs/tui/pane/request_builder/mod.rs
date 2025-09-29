use std::io::Stdout;

use crate::{
    app_project::models::{BodyContent, KeyValueParam},
    programs::tui::{
        common::{component::WithHistory, Interactive, UiComposedElement},
        config::{keybinding, Config},
        event_handler::{AppMessage, Events},
    },
};
use arboard::Clipboard;
use body_editor::BodyEditor;
use params_table::ParamsTable;
use ratatui::{
    layout::{Constraint, Layout, Margin, Rect},
    prelude::CrosstermBackend,
    style::{Style, Stylize},
    widgets::{Block, BorderType, Borders, Tabs},
    Terminal,
};

use super::{action::PaneAction, ElementFocus};

mod body_binary;
mod body_editor;
mod body_form;
mod body_text;
mod params_table;

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
    params_table: ParamsTable,
    headers_table: ParamsTable,
    body_editor: BodyEditor,
}

impl RequestEditorComponent {
    pub fn new(config: &Config) -> Self {
        Self {
            tab: Tab::Params,
            header_area: Rect::default(),
            render_area: Rect::default(),
            params_table: ParamsTable::new(),
            headers_table: ParamsTable::new(),
            body_editor: BodyEditor::new(config),
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
        self.body_editor.set_state(body);
    }

    pub fn get_data(&mut self) -> (Vec<KeyValueParam>, Vec<KeyValueParam>, BodyContent) {
        (
            self.params_table.get_data(),
            self.headers_table.get_data(),
            self.body_editor.get_data(),
        )
    }

    pub fn change_tab(&mut self, is_next: bool) {
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
                if !self.body_editor.is_input_focus() {
                    if !is_next {
                        self.tab = Tab::Headers;
                    }
                }
            }
            Tab::Params => {
                if !self.params_table.is_editing() {
                    if is_next {
                        self.tab = Tab::Headers;
                    }
                }
            }
        }
    }
}

impl<'params> UiComposedElement<'params> for RequestEditorComponent {
    type Params = (ElementFocus, &'params Config);

    fn set_area(&mut self, area: Rect, _viewport_area: Rect) {
        let main_areas = Layout::vertical([Constraint::Length(2), Constraint::Fill(50)])
            .split(area.inner(Margin::new(1, 1)));

        self.render_area = area;
        self.header_area = main_areas[0];
        self.params_table.set_area(main_areas[1], _viewport_area);
        self.headers_table.set_area(main_areas[1], _viewport_area);
        self.body_editor.set_area(main_areas[1], _viewport_area);
    }

    fn draw(&self, (focus, config): Self::Params, frame: &mut ratatui::Frame) {
        let border_style = if focus == ElementFocus::RequestBuilder {
            Style::default().fg(config.theme.border_focus)
        } else {
            Style::default().fg(config.theme.border)
        };

        let border_type = if focus == ElementFocus::RequestBuilder {
            BorderType::Thick
        } else {
            BorderType::Plain
        };

        let block = Block::bordered()
            .border_style(border_style)
            .border_type(border_type);
        frame.render_widget(block, self.render_area);

        let tabs = Tabs::new([
            format!(
                " {} ({})",
                Tab::Params.as_ref().fg(config.theme.tab),
                self.params_table.len_items()
            ),
            format!(
                " {} ({})",
                Tab::Headers.as_ref().fg(config.theme.tab),
                self.headers_table.len_items()
            ),
            format!(" {} ", Tab::Body.as_ref().fg(config.theme.tab)),
        ])
        .select(self.tab.as_idx())
        .block(
            Block::new()
                .borders(Borders::BOTTOM)
                .border_style(border_style)
                .border_type(border_type),
        )
        .highlight_style(Style::default().fg(config.theme.tab_highlight));

        frame.render_widget(tabs, self.header_area);

        match self.tab {
            Tab::Headers => {
                self.headers_table.draw(config, frame);
            }
            Tab::Body => {
                // self.body_editor_component.draw(config, frame);
                self.body_editor.draw(config, frame);
            }
            Tab::Params => {
                self.params_table.draw(config, frame);
            }
        }
    }

    fn draw_overlay(&self, (_, config): Self::Params, frame: &mut ratatui::Frame) {
        match self.tab {
            Tab::Headers => self.headers_table.draw_overlay(config, frame),
            Tab::Body => self.body_editor.draw_overlay(config, frame),
            Tab::Params => self.params_table.draw_overlay(config, frame),
        }
    }
}

impl<'params> Interactive<'params> for RequestEditorComponent {
    type Effect = PaneAction;

    type Params = (
        &'params Config,
        &'params mut Events<AppMessage>,
        &'params mut Terminal<CrosstermBackend<Stdout>>,
        &'params mut Clipboard,
    );

    fn is_visible_overlay(&self) -> bool {
        match self.tab {
            Tab::Headers => self.headers_table.is_visible_overlay(),
            Tab::Body => self.body_editor.is_visible_overlay(),
            Tab::Params => self.params_table.is_visible_overlay(),
        }
    }

    fn handle_key(
        &mut self,
        (config, events, terminal, clipboard): Self::Params,
        key: crossterm::event::KeyEvent,
    ) -> Self::Effect {
        if let Some(action) = config.keymap.match_global_action(key) {
            let block_focus_nav = match self.tab {
                Tab::Headers => {
                    self.headers_table.is_visible_overlay() | self.headers_table.is_editing()
                }
                Tab::Body => {
                    self.body_editor.is_visible_overlay() | self.body_editor.is_input_focus()
                }
                Tab::Params => {
                    self.params_table.is_visible_overlay() | self.params_table.is_input_focus()
                }
            };

            match action {
                keybinding::GlobalKeyAction::NextTab => {
                    self.change_tab(true);
                }
                keybinding::GlobalKeyAction::PreviousTab => {
                    self.change_tab(false);
                }
                keybinding::GlobalKeyAction::NextFocus if !block_focus_nav => {
                    return PaneAction::NextFocus;
                }
                keybinding::GlobalKeyAction::PreviousFocus if !block_focus_nav => {
                    return PaneAction::PreviousFocus;
                }

                _ => {}
            }
        }

        match self.tab {
            Tab::Headers => {
                self.headers_table.handle_key((config, clipboard), key);
            }
            Tab::Body => {
                self.body_editor
                    .handle_key((config, events, terminal, clipboard), key);
            }
            Tab::Params => {
                self.params_table.handle_key((config, clipboard), key);
            }
        }

        PaneAction::Noop
    }
}

impl WithHistory for RequestEditorComponent {
    fn undo(&mut self) {
        match self.tab {
            Tab::Headers => self.headers_table.undo(),
            Tab::Body => self.body_editor.undo(),
            Tab::Params => self.params_table.undo(),
        }
    }

    fn redo(&mut self) {
        match self.tab {
            Tab::Headers => self.headers_table.redo(),
            Tab::Body => self.body_editor.redo(),
            Tab::Params => self.params_table.redo(),
        }
    }
}
