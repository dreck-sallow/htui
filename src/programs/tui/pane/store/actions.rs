use std::collections::HashMap;

use crate::store::models::{BodyContent, CollectionsModel, HttpMethod, RequestModel};

pub enum Action {
    NextFocus,
    PreviousFocus,

    // Collection actions
    CreateCollection(String),
    InsertCollection {
        idx: usize,
        collection: CollectionsModel,
    },
    DeleteCollection {
        idx: usize,
    },
    EditCollectionName {
        idx: usize,
        new_name: String,
    },

    // Request actions
    CreateRequest {
        coll_idx: usize,
        name: String,
    },
    InsertRequest {
        idx: (usize, usize),
        request: RequestModel,
    },
    DeleteRequest((usize, usize)),
    EditRequestMethod {
        idx: (usize, usize),
        method: HttpMethod,
    },
    EditRequestUrl {
        idx: (usize, usize),
        url: String,
    },
    EditRequestHeaders {
        idx: (usize, usize),
        headers: HashMap<String, String>,
    },
    EditRequestBody {
        idx: (usize, usize),
        body: BodyContent,
    },
    SetCurrentRequest(Option<(usize, usize)>),
}

impl Action {
    pub fn is_delete_or_insert(&self) -> bool {
        match self {
            Action::CreateCollection(_) => true,
            Action::InsertCollection { .. } => true,
            Action::DeleteCollection { .. } => true,
            Action::EditCollectionName { .. } => true,
            Action::CreateRequest { .. } => true,
            Action::InsertRequest { .. } => true,
            Action::DeleteRequest(_) => true,
            _ => false,
        }
    }

    pub fn is_set_current_request(&self) -> bool {
        if let Self::SetCurrentRequest(_) = self {
            true
        } else {
            false
        }
    }
}
