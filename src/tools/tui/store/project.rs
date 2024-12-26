use serde::{Deserialize, Serialize};

/// Structure used for serialize in store mapping-file (store.toml)
#[derive(Deserialize, Serialize, Clone)]
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
    pub(crate) name: String,
    pub(crate) id: String,
}

#[derive(Deserialize, Serialize, Default, Clone)]
pub struct Project {
    pub name: String,
    pub collections: Vec<Collection>,
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
