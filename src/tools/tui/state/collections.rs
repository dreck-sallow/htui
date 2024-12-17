#[derive(Default)]
pub struct CollectionsState {
    collections: Vec<Collection>,
    cursor: CollectionCursor,
}

type CollectionCursor = (Option<usize>, Option<usize>);

#[derive(Default)]
pub struct Collection {
    name: String,
    requests: Vec<CollectionRequest>,
}

impl Collection {
    pub fn with_name<R>(name: R) -> Self
    where
        R: Into<String>,
    {
        Self {
            name: name.into(),
            requests: Vec::new(),
        }
    }
    pub fn append<R>(&mut self, request_name: R)
    where
        R: Into<String>,
    {
        self.requests.push(CollectionRequest {
            name: request_name.into(),
        })
    }
}

pub struct CollectionRequest {
    name: String,
}

impl CollectionRequest {
    pub fn new<R>(name: R) -> Self
    where
        R: Into<String>,
    {
        Self { name: name.into() }
    }
}

impl CollectionsState {
    pub fn with_collections(collections: Vec<Collection>) -> Self {
        let cursor = if collections.is_empty() {
            (None, None)
        } else {
            (Some(0), None)
        };

        Self {
            collections,
            cursor,
        }
    }
    pub fn append_collection(&mut self, collection: Collection) {
        self.collections.push(collection);

        if self.collections.len() == 1 {
            self.cursor = (Some(0), None);
        }
    }

    pub fn append_request(&mut self, request: CollectionRequest) {
        if let (Some(collection_idx), _) = self.cursor {
            self.collections[collection_idx].requests.push(request);

            if self.collections[collection_idx].requests.len() == 1 {
                self.cursor = (Some(collection_idx), Some(0));
            }
        }
    }

    pub fn get_current_collection(&self) -> Option<&Collection> {
        if let ((Some(idx), _)) = self.cursor {
            return self.collections.get(idx);
        }
        None
    }

    pub fn get_current_request(&self) -> Option<&CollectionRequest> {
        if let ((Some(coll_idx), Some(req_idx))) = self.cursor {
            return self.collections[coll_idx].requests.get(req_idx);
        }
        None
    }

    pub fn next(&mut self) {
        match self.cursor {
            (Some(collection_index), Some(request_index)) => {
                let collection = &self.collections[collection_index];

                match next_index(&collection.requests, request_index) {
                    Some(next) => {
                        self.cursor = (Some(collection_index), Some(next));
                    }
                    None => {
                        if let Some(next) = next_index(&self.collections, collection_index) {
                            self.cursor = (Some(next), None);
                        }
                    }
                }
            }
            (Some(collection_index), None) => {
                let collection = &self.collections[collection_index];

                println!(
                    "collection length: {}, index: {}",
                    self.collections.len(),
                    collection_index,
                );

                if !collection.requests.is_empty() {
                    self.cursor = (Some(collection_index), Some(0))
                } else if let Some(next) = next_index(&self.collections, collection_index) {
                    self.cursor = (Some(next), None);
                }
            }
            _ => {
                if !self.collections.is_empty() {
                    self.cursor = (Some(0), None);
                }
            }
        }
    }
    pub fn remove(&mut self) {}
}

fn next_index<T>(vec: &Vec<T>, index: usize) -> Option<usize> {
    if index < (vec.len() - 1) {
        return Some(index + 1);
    }
    None
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn no_indices_in_empty() {
//         let mut collection_state = CollectionsState::default();
//         collection_state.next();
//         collection_state.next();
//         collection_state.next();

//         assert_eq!(
//             collection_state.cursor,
//             (None, None),
//             "state should be none when no have collections"
//         );
//     }

//     #[test]
//     fn collection_first_index() {
//         let mut collection_state = CollectionsState::default();
//         collection_state.append_collection(Collection::default());

//         assert_eq!(
//             collection_state.cursor,
//             (Some(0), None),
//             "state should select first collection inserted"
//         );

//         collection_state.next();
//         collection_state.next();
//         collection_state.next();
//         collection_state.next();
//         collection_state.next();

//         assert_eq!(
//             collection_state.cursor,
//             (Some(0), None),
//             "not should change cursor in one collection"
//         );
//     }

//     #[test]
//     fn collection_second_index() {
//         let mut collection_state = CollectionsState::default();
//         collection_state.append_collection(Collection::default());
//         collection_state.append_collection(Collection::default());

//         assert_eq!(collection_state.cursor, (Some(0), None), "append coll: 2");

//         collection_state.next();
//         collection_state.next();

//         assert_eq!(
//             collection_state.cursor,
//             (Some(1), None),
//             "call next: coll idx 2"
//         );
//     }

//     #[test]
//     fn append_req_in_empty_collections() {
//         let mut state = CollectionsState::default();
//         state.append_request(CollectionRequest::new("empty"));

//         assert_eq!(state.cursor, (None, None), "append req 1 in coll: 0")
//     }

//     #[test]
//     fn append_req_in_collection() {
//         let mut state =
//             CollectionsState::with_collections([Collection::with_name("coll_1")].into());

//         state.append_request(CollectionRequest::new("coll_1_req_1"));
//         assert_eq!(state.cursor, (Some(0), Some(0)), "append req 1 in coll 1");

//         state.append_request(CollectionRequest::new("coll_1_req_2"));
//         assert_eq!(state.cursor, (Some(0), Some(0)), "append req 2 in coll 1")
//     }

//     #[test]
//     fn get_empty_collection_request() {
//         let mut state = CollectionsState::default();

//         let collection_name = state.get_current_collection().map(|coll| &coll.name);
//         assert_eq!(collection_name, None);

//         let request_name = state.get_current_request().map(|coll| &coll.name);
//         assert_eq!(request_name, None);
//     }

//     #[test]
//     fn get_2_collection() {
//         let mut state = CollectionsState::with_collections(
//             [
//                 Collection::with_name("coll_1"),
//                 Collection::with_name("coll_2"),
//             ]
//             .into(),
//         );

//         state.next();

//         state.append_request(CollectionRequest::new("coll_1_req_1"));
//         state.append_request(CollectionRequest::new("coll_1_req_2"));

//         let collection_name = state.get_current_collection().map(|coll| &coll.name);
//         assert_eq!(collection_name, Some(&String::from("coll_2")));

//         let req_name = state.get_current_request().map(|req| &req.name);
//         assert_eq!(req_name, Some(&String::from("coll_1_req_1")));

//         state.next();

//         let req_name = state.get_current_request().map(|req| &req.name);
//         assert_eq!(req_name, Some(&String::from("coll_1_req_2")));
//     }
// }
