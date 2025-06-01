use collections::{Collections, Item};
use ratatui::{layout::Rect, widgets::Block, Frame};
// use tui_tree_widget::Tree;

use state::CollectionsState;

use crate::store::models::{CollectionsModel, ProjectModel, RequestModel};

mod collections;
mod state;

pub struct CollectionsView {
    state: CollectionsState<String>,
}

impl CollectionsView {
    pub fn new() -> Self {
        Self {
            state: CollectionsState::new(),
        }
    }

    pub fn insert_collection(&mut self, collection: &CollectionsModel) {
        let children = collection
            .requests()
            .iter()
            .map(|req| req.id().to_string())
            .collect();

        self.state
            .add_collection((collection.id().to_string(), children));
    }

    pub fn insert_request(&mut self, collection: &CollectionsModel, request: &RequestModel) {
        self.state
            .add_request(collection.id().to_string(), request.id().to_string());
    }

    pub fn render(&self, project: &ProjectModel, frame: &mut Frame, area: Rect) {
        let items = project
            .collections()
            .iter()
            .map(|coll| {
                let mut itm = Item::new(coll.name());

                for req in coll.requests() {
                    itm.add_child(Item::new(req.name()));
                }

                itm
            })
            .collect();
        let collections = Collections::default()
            .set_items(items)
            .set_block(Block::bordered().title(" Collections "))
            .set_openeds(self.state.openeds())
            .set_idx(self.state.idx());

        frame.render_widget(collections, area);
    }
}
