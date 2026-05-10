use collections::CollectionsList;
use responses::Responses;

pub mod collections;
pub mod param_item;
pub mod responses;
pub mod table;

#[derive(PartialEq, Eq)]
pub enum SectionFocus {
    Collections,
    RequestBar,
    RequestBuilder,
    ResponseViewer,
}

pub struct PaneState {
    pub(crate) focus: SectionFocus,
    pub(crate) collections: CollectionsList,
    // pub(crate) environments: Environments,
    // pub(crate) selected_env_context: Option<usize>,
    pub(crate) responses: Responses,
}

impl PaneState {
    pub fn new(list: CollectionsList) -> Self {
        Self {
            focus: SectionFocus::Collections,
            collections: list,
            responses: Responses::new(),
        }
    }
}
