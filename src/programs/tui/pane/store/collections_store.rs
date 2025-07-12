use crate::store::models::{CollectionsModel, RequestModel};

/// bound behavior of the collections_store
/// useful for define the methods to use as list state
pub trait MutableList {
    fn insert_collection(&mut self, idx: usize, collection: CollectionsModel);
    fn insert_request(&mut self, collection_idx: usize, idx: usize, req: RequestModel);

    fn remove_collection(&mut self, idx: usize) -> Option<CollectionsModel>;
    fn remove_request(&mut self, idx: (usize, usize)) -> Option<RequestModel>;

    fn get_collection(&self, idx: usize) -> Option<&CollectionsModel>;
    fn get_request(&self, idx: (usize, usize)) -> Option<&RequestModel>;

    fn get_collection_mut(&mut self, idx: usize) -> Option<&mut CollectionsModel>;
    fn get_request_mut(&mut self, idx: (usize, usize)) -> Option<&mut RequestModel>;

    fn edit_request<F: FnMut(&mut RequestModel)>(&mut self, idx: (usize, usize), f: F);
}

pub type Inner = Vec<CollectionsModel>;

pub struct CollectionsStore {
    inner: Inner,
}

impl CollectionsStore {
    pub fn new(collections: Vec<CollectionsModel>) -> Self {
        Self { inner: collections }
    }

    pub fn list(&self) -> &Inner {
        &self.inner
    }
}

impl MutableList for CollectionsStore {
    fn insert_collection(&mut self, idx: usize, collection: CollectionsModel) {
        self.inner.insert(idx, collection);
    }

    fn insert_request(&mut self, collection_idx: usize, idx: usize, req: RequestModel) {
        if let Some(coll) = self.inner.get_mut(collection_idx) {
            coll.requests.insert(idx, req);
        }
    }

    fn remove_collection(&mut self, idx: usize) -> Option<CollectionsModel> {
        if idx < self.inner.len() {
            Some(self.inner.remove(idx))
        } else {
            None
        }
    }

    fn remove_request(&mut self, (i, sub_i): (usize, usize)) -> Option<RequestModel> {
        if let Some(coll) = self.inner.get_mut(i) {
            if sub_i < coll.requests.len() {
                return Some(coll.requests.remove(sub_i));
            }
        }

        None
    }

    fn get_collection(&self, idx: usize) -> Option<&CollectionsModel> {
        self.inner.get(idx)
    }

    fn get_request(&self, (i, sub_i): (usize, usize)) -> Option<&RequestModel> {
        if let Some(coll) = self.inner.get(i) {
            return coll.requests.get(sub_i);
        }
        None
    }

    fn get_collection_mut(&mut self, idx: usize) -> Option<&mut CollectionsModel> {
        self.inner.get_mut(idx)
    }

    fn get_request_mut(&mut self, idx: (usize, usize)) -> Option<&mut RequestModel> {
        self.inner
            .get_mut(idx.0)
            .and_then(|coll| coll.requests.get_mut(idx.1))
    }

    fn edit_request<F: FnMut(&mut RequestModel)>(&mut self, idx: (usize, usize), mut f: F) {
        if let Some(req) = self.get_request_mut(idx) {
            f(req)
        }
    }
}
