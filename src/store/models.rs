use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

pub fn time_as_id() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string()
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ProjectModel {
    id: String,
    name: String,
    collections: Vec<CollectionsModel>,
}

impl ProjectModel {
    pub fn new(name: String) -> Self {
        Self {
            id: time_as_id(),
            name,
            collections: Vec::new(),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn collections(&self) -> &[CollectionsModel] {
        &self.collections
    }

    pub fn add_collection(&mut self, collection: CollectionsModel) {
        self.collections.push(collection);
    }

    pub fn add_request_by_i(&mut self, i: usize, request: RequestModel) {
        if let Some(coll) = self.collections.get_mut(i) {
            coll.requests.push(request);
        }
    }

    pub fn collection_by_idx(&self, i: usize) -> Option<&CollectionsModel> {
        self.collections.get(i)
    }

    pub fn request_by_idx(&self, (i, sub_i): (usize, usize)) -> Option<&RequestModel> {
        self.collections.get(i).and_then(|c| c.requests.get(sub_i))
    }
}

impl Default for ProjectModel {
    fn default() -> Self {
        let id = time_as_id();
        Self {
            id: id.clone(),
            name: id,
            collections: Vec::new(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CollectionsModel {
    id: String,
    name: String,
    requests: Vec<RequestModel>,
}

impl CollectionsModel {
    pub fn new(name: String) -> Self {
        Self {
            id: time_as_id(),
            name,
            requests: Vec::new(),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn requests(&self) -> &[RequestModel] {
        &self.requests
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RequestModel {
    id: String,
    name: String,
    headers: HashMap<String, String>,
    method: String, // TODO: define a http method as method model
}

impl RequestModel {
    pub fn new(name: String, method: String) -> Self {
        Self {
            id: time_as_id(),
            name,
            headers: HashMap::default(),
            method,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
