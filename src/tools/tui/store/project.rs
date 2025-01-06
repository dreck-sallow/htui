use std::time;

use serde::{Deserialize, Serialize};

/// Structure used for serialize in store mapping-file (store.toml)
#[derive(Deserialize, Serialize, Clone, Default)]
pub struct StoreMapping {
    pub projects: Vec<StoreMappingItem>,
}

impl StoreMapping {
    // pub fn insert(&mut self, item: StoreMappingItem) {
    //     let find_item = self
    //         .projects
    //         .iter()
    //         .enumerate()
    //         .find(|(_i, itm)| itm.id == item.id);

    //     match find_item {
    //         Some((i, _)) => {
    //             self.projects[i] = item;
    //         }
    //         None => {
    //             self.projects.push(item);
    //         }
    //     }
    // }

    pub fn find_by_name(&self, name: &str) -> Option<&StoreMappingItem> {
        self.projects.iter().find(|itm| itm.name == name)
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct StoreMappingItem {
    name: String,
    id: String,
}

impl StoreMappingItem {
    pub fn new<N: Into<String>>(name: N) -> Self {
        // TODO: check if we can move to utils file
        let current_millis = time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .unwrap()
            .as_millis();

        Self {
            name: name.into(),
            id: current_millis.to_string(),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Deserialize, Serialize, Default, Clone)]
pub struct Project {
    pub name: String,
    pub collections: Vec<Collection>,
}

impl Project {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            collections: Vec::new(),
        }
    }
}

#[derive(Deserialize, Serialize, Default, Clone)]
pub struct Collection {
    pub name: String,
    pub requests: Vec<Request>,
}

#[derive(Deserialize, Serialize, Default, Clone)]
pub struct Request {
    pub name: String,
    pub method: String,
    pub url: String,
}
