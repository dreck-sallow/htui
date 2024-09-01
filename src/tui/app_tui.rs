use ratatui::{
    crossterm::event::{self, KeyEventKind},
    layout::{Constraint, Layout},
    symbols::border,
    widgets::Block,
    Frame,
};

use crate::store;

use super::AppTuiResult;

#[derive(Default)]
pub struct AppTuistate {
    current_url: String,
    collections: Vec<store::entity::Collection>,
}

pub struct AppTui {
    state: AppTuistate,
    is_exit: bool,
}

impl AppTui {
    pub fn new() -> Self {
        Self {
            state: AppTuistate::default(),
            is_exit: false,
        }
    }

    pub fn run(&mut self, frame: &mut Frame) -> AppTuiResult {
        let block = Block::bordered()
            .title("HTUI Application for terminal")
            .border_set(border::THICK);

        let [sidebar_section, main_section] =
            Layout::horizontal([Constraint::Percentage(20), Constraint::Fill(1)])
                .areas(frame.size());

        let [url_section, actions_section] =
            Layout::vertical([Constraint::Percentage(10), Constraint::Fill(1)]).areas(main_section);

        let [body_section, response_section] =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(actions_section);

        frame.render_widget(block, sidebar_section); // REnder sudebar

        //Render url bar
        let url_bar = Block::bordered().title("Enter an URL");
        frame.render_widget(url_bar, url_section);

        // Body section:
        let body_block = Block::bordered().title("Write your request");
        frame.render_widget(body_block, body_section);

        // response section:
        let response_block = Block::bordered().title("Response block");
        frame.render_widget(response_block, response_section);
        Ok(())
    }

    fn draw_collections(&self) -> Block {
        let sidebar_block = Block::bordered().title(format!(
            "Collections list ({})",
            self.state.collections.len()
        ));

        return sidebar_block;
    }

    pub fn is_finish(&self) -> bool {
        self.is_exit
    }

    pub fn handle_events(&mut self) -> AppTuiResult {
        match event::read()? {
            event::Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    event::KeyCode::Char('q') => self.is_exit = true,
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }
}
