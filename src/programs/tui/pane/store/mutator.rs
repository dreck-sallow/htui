use std::mem;

use crate::{
    programs::tui::pane::{state::ElementFocus, store::collections_store::MutableList},
    store::models::{CollectionsModel, RequestModel},
};

use super::{actions::Action, PaneStore};

pub struct MutationsHistoryV2 {
    stack: Vec<Action>,
    cursor: Option<usize>,
}

impl MutationsHistoryV2 {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            cursor: None,
        }
    }

    pub fn apply_from_list(&mut self, actions: Vec<Action>, store: &mut PaneStore) {
        for action in actions {
            if let Some(dif_action) = apply_action(action, store) {
                self.stack.push(dif_action);

                self.cursor = match self.cursor {
                    Some(i) => Some(i + 1),
                    None => Some(0),
                };
            }
        }
    }

    fn apply_action_by_idx(&mut self, cursor: usize, store: &mut PaneStore) {
        // WARN: I use a weird-uggly-bad-terrible-... action for avoid remove or pass a reference to apply_action
        // I will search for a better solution

        // A temp-fake action onyl for store the cursor action
        let mut action = Action::NextFocus;
        mem::swap(&mut action, &mut self.stack[cursor]);
        let new_action = apply_action(action, store).unwrap();
        self.stack[cursor] = new_action;
    }

    pub fn go_forward(&mut self, store: &mut PaneStore) {
        match self.cursor {
            Some(cursor) => {
                if cursor + 1 < self.stack.len() {
                    self.apply_action_by_idx(cursor + 1, store);
                    self.cursor = Some(cursor + 1);
                }
            }
            None => {
                if !self.stack.is_empty() {
                    self.apply_action_by_idx(0, store);
                    self.cursor = Some(0);
                }
            }
        }
    }

    pub fn go_back(&mut self, store: &mut PaneStore) {
        if let Some(cursor) = self.cursor {
            self.apply_action_by_idx(cursor, store);

            self.cursor = if cursor == 0 { None } else { Some(cursor - 1) };
        }
    }
}

/// Apply an action to store, and return an action if its necesary track for history (for undo actions)
fn apply_action(action: Action, store: &mut PaneStore) -> Option<Action> {
    match action {
        Action::CreateCollection(name) => {
            let collection = CollectionsModel::new(name);

            let inserted_index = store.collections.list().len();
            store
                .collections
                .insert_collection(inserted_index, collection);

            Some(Action::DeleteCollection {
                idx: inserted_index,
            })
        }
        Action::InsertCollection { idx, collection } => {
            store.collections.insert_collection(idx, collection);

            Some(Action::DeleteCollection { idx: idx })
        }
        Action::DeleteCollection { idx } => {
            let collection = store.collections.remove_collection(idx).unwrap();
            Some(Action::InsertCollection { idx, collection })
        }
        Action::EditCollectionName { idx, new_name } => {
            let collection = store.collections.get_collection_mut(idx).unwrap();
            let previous_name = collection.name.to_string();

            collection.name = new_name;

            Some(Action::EditCollectionName {
                idx,
                new_name: previous_name,
            })
        }
        Action::CreateRequest { coll_idx, name } => {
            let collection = store.collections.get_collection(coll_idx).unwrap();
            let req = RequestModel::new(name);
            let inserted_i = collection.requests.len();

            store.collections.insert_request(coll_idx, inserted_i, req);

            Some(Action::DeleteRequest((coll_idx, inserted_i)))
        }
        Action::InsertRequest { idx, request } => {
            store.collections.insert_request(idx.0, idx.1, request);

            Some(Action::DeleteRequest(idx))
        }
        Action::DeleteRequest(idx) => {
            let request = store.collections.remove_request(idx).unwrap();
            store.collections.remove_request(idx);

            Some(Action::InsertRequest { idx: idx, request })
        }
        Action::EditRequestMethod { idx, method } => {
            let request = store.collections.get_request_mut(idx).unwrap();
            let previous_method = request.method().clone();

            store.collections.edit_request(idx, |req| {
                req.set_method(method);
            });

            Some(Action::EditRequestMethod {
                idx,
                method: previous_method,
            })
        }
        Action::EditRequestUrl { idx, url } => {
            let request = store.collections.get_request_mut(idx).unwrap();
            let previous_url = request.url().to_string();

            store.collections.edit_request(idx, |req| {
                req.set_url(url.clone());
            });

            Some(Action::EditRequestUrl {
                idx,
                url: previous_url,
            })
        }
        Action::EditRequestHeaders { idx, headers } => {
            let request = store.collections.get_request_mut(idx).unwrap();
            let previous_headers = request.headers_map().clone();

            store.collections.edit_request(idx, |req| {
                req.set_headers(headers.clone());
            });

            Some(Action::EditRequestHeaders {
                idx,
                headers: previous_headers,
            })
        }
        Action::EditRequestBody { idx, body } => {
            let request = store.collections.get_request_mut(idx).unwrap();
            let previous_body = request.body().clone();

            store.collections.edit_request(idx, |req| {
                req.set_body(body.clone());
            });

            Some(Action::EditRequestBody {
                idx,
                body: previous_body,
            })
        }

        Action::NextFocus => {
            let focus = match store.focus {
                ElementFocus::Collections => ElementFocus::MethodUrlBar,
                ElementFocus::MethodUrlBar => ElementFocus::RequestBuilder,
                ElementFocus::RequestBuilder => ElementFocus::ResponseViewer,
                ElementFocus::ResponseViewer => ElementFocus::Collections,
            };

            store.set_focus(focus);

            None
        }
        Action::PreviousFocus => {
            let focus = match store.focus {
                ElementFocus::Collections => ElementFocus::ResponseViewer,
                ElementFocus::MethodUrlBar => ElementFocus::Collections,
                ElementFocus::RequestBuilder => ElementFocus::MethodUrlBar,
                ElementFocus::ResponseViewer => ElementFocus::RequestBuilder,
            };
            store.set_focus(focus);
            None
        }
        Action::SetCurrentRequest(idx) => {
            store.set_current_idx(idx);
            None
        }
    }
}
