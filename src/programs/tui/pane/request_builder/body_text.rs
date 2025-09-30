use std::io::{stdout, Stdout};

use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{layout::Rect, prelude::CrosstermBackend, Terminal};

use crate::{
    app_project::models::BodyContent,
    programs::tui::{
        common::{Interactive, UiComposedElement},
        config::{keybinding, Config},
        event_handler::{AppMessage, Events},
        pane::text_editor::TextEditor,
    },
};

pub struct BodyTextEditor {
    render_area: Rect,
    editor: TextEditor,
}

impl BodyTextEditor {
    pub fn new() -> Self {
        Self {
            render_area: Rect::default(),
            editor: TextEditor::new(true),
        }
    }

    pub fn body_model(&self) -> BodyContent {
        BodyContent::Text(self.editor.lines().join("\n"))
    }

    pub fn set_data(&mut self, text: &str) {
        self.editor.clean_lines();
        self.editor.insert_str(text);
    }

    pub fn render_area(&self) -> Rect {
        self.render_area
    }
}

impl<'params> UiComposedElement<'params> for BodyTextEditor {
    type Params = ();

    fn set_area(&mut self, area: ratatui::prelude::Rect, _viewport_area: ratatui::prelude::Rect) {
        self.render_area = area;
    }

    fn draw(&self, _params: Self::Params, frame: &mut ratatui::Frame) {
        frame.render_widget(&self.editor, self.render_area);
    }
}

impl<'params> Interactive<'params> for BodyTextEditor {
    type Effect = ();
    type Params = (
        &'params Config,
        &'params mut Events<AppMessage>,
        &'params mut Terminal<CrosstermBackend<Stdout>>,
        // &'params mut Clipboard,
    );

    fn is_input_focus(&self) -> bool {
        self.editor.mode().is_write_mode()
    }

    fn handle_key(
        &mut self,
        (config, events, terminal): Self::Params,
        key: crossterm::event::KeyEvent,
    ) -> Self::Effect {
        let mut consumed = false;
        if let Some(action) = config.keymap.match_request_builder_action(key) {
            match action {
                keybinding::RequestBuilderKeyAction::OpenEditor => {
                    events.stop();
                    disable_raw_mode().unwrap();
                    stdout().execute(LeaveAlternateScreen).unwrap();

                    self.editor.open_in_editor();

                    enable_raw_mode().unwrap();
                    stdout().execute(EnterAlternateScreen).unwrap();
                    let _ = terminal.clear();
                    events.run();
                    consumed = true;
                }
                _ => {}
            }

            return;
        }

        if !consumed {
            self.editor.handle_key(key);
        }
    }
}
