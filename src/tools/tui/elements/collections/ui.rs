use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::Span,
    widgets::{Block, Borders},
    Frame,
};

use super::state::{CollectionItem, CollectionState};

pub struct CollectionsUi {}

impl CollectionsUi {
    // TODO: get the theme config and sert the reference
    pub fn new() -> Self {
        Self {}
    }

    pub fn render(frame: &mut Frame, area: Rect, state: &CollectionState) {
        let tree_block = Block::default()
            .title(" Collections ")
            .borders(Borders::ALL);

        frame.render_widget(tree_block, area);
    }

    pub fn render_collection(collection: &CollectionItem) -> Span {
        return Span::default().style(Style::new().red().on_red());
    }
}
