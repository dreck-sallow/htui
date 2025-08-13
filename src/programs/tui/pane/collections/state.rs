use std::{collections::HashSet, fmt::Debug, ops::Not};

use crate::app_project::models::{CollectionsModel, RequestModel};

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

pub struct CollectionsState {
    collections: Vec<CollectionsModel>,
    idx: Idx,
    opened: HashSet<usize>,
    selected_request_idx: Option<(usize, usize)>,
}

impl CollectionsState {
    // pub fn new() -> Self {
    //     Self {
    //         collections: Vec::new(),
    //         idx: Idx::default(),
    //         opened: HashSet::new(),
    //     }
    // }

    pub fn from_list(collections: Vec<CollectionsModel>) -> Self {
        Self {
            idx: if collections.is_empty() {
                Idx::None
            } else {
                Idx::Parent(0)
            },
            collections,
            selected_request_idx: None,
            opened: HashSet::new(),
        }
    }

    // TODO: delete?
    pub fn collections(&self) -> &[CollectionsModel] {
        &self.collections
    }

    pub fn openeds(&self) -> HashSet<usize> {
        self.opened.clone()
    }

    pub fn idx(&self) -> Idx {
        self.idx
    }

    pub fn selected_request_idx(&self) -> Option<(usize, usize)> {
        self.selected_request_idx
    }

    pub fn select_request_idx(&mut self, idx: Option<(usize, usize)>) {
        self.selected_request_idx = idx;
    }

    pub fn next_collection(&mut self) {
        let idx = match self.idx {
            Idx::None => self.collections.is_empty().not().then_some(0),
            Idx::Parent(i) | Idx::Child(i, _) => (i < self.collections.len() - 1).then_some(i + 1),
        };

        if let Some(i) = idx {
            self.idx = Idx::Parent(i);
        }
    }

    pub fn next_request(&mut self) {
        let child_idx = match self.idx {
            Idx::None => self
                .collections
                .first()
                .and_then(|coll| (!coll.requests.is_empty()).then_some((0, 0))),
            Idx::Parent(i) => {
                if self.opened.contains(&i) {
                    let coll = &self.collections[i];
                    (!coll.requests.is_empty()).then_some((i, 0))
                } else {
                    None
                }
            }
            Idx::Child(i, sub_i) => {
                let coll = &self.collections[i];
                (sub_i < coll.requests.len() - 1).then_some((i, sub_i + 1))
            }
        };

        if let Some((i, sub_i)) = child_idx {
            self.idx = Idx::Child(i, sub_i);
        }
    }

    pub fn next(&mut self) {
        match self.idx {
            Idx::None => self.next_collection(),
            _ => {
                let previous_idx = self.idx;
                self.next_request();

                if self.idx == previous_idx {
                    self.next_collection();
                }
            }
        }
    }

    pub fn prev_collection(&mut self) {
        let idx = match self.idx {
            Idx::None => None,
            Idx::Parent(i) => (i > 0).then(|| i - 1),
            Idx::Child(i, _) => Some(i),
        };

        if let Some(i) = idx {
            self.idx = Idx::Parent(i);
        }
    }

    pub fn prev_request(&mut self) {
        let idx = match self.idx {
            Idx::None => None,
            Idx::Parent(i) => i
                .checked_sub(1)
                .and_then(|prev_i| self.opened.contains(&prev_i).then_some(prev_i))
                .and_then(|prev_i| Some((prev_i, &self.collections[prev_i])))
                .and_then(|(prev_i, coll)| {
                    (!coll.requests.is_empty()).then(|| (prev_i, coll.requests.len() - 1))
                }),
            Idx::Child(i, sub_i) => (sub_i > 0).then(|| (i, sub_i - 1)),
        };

        if let Some((i, sub_i)) = idx {
            self.idx = Idx::Child(i, sub_i);
        }
    }

    pub fn prev(&mut self) {
        match self.idx {
            Idx::None => {}
            _ => {
                let previous_idx = self.idx;
                self.prev_request();

                if self.idx == previous_idx {
                    self.prev_collection();
                }
            }
        }
    }

    pub fn open_collection(&mut self, only_parent: bool) {
        match self.idx {
            Idx::Parent(i) => {
                self.opened.insert(i);
            }
            Idx::Child(i, _) => {
                if !only_parent {
                    self.opened.insert(i);
                }
            }
            _ => {}
        }
    }

    pub fn close_collection(&mut self, only_parent: bool) {
        match self.idx {
            Idx::Parent(i) => {
                self.opened.remove(&i);
            }
            Idx::Child(i, _) => {
                if !only_parent {
                    self.opened.remove(&i);
                }
            }
            _ => {}
        }
    }

    pub fn current_request(&self) -> Option<&RequestModel> {
        match self.idx {
            Idx::None => None,
            Idx::Parent(_) => None,
            Idx::Child(i, sub_i) => self.collections[i].requests.get(sub_i),
        }
    }
}

impl MutableList for CollectionsState {
    fn insert_collection(&mut self, idx: usize, collection: CollectionsModel) {
        self.collections.insert(idx, collection);

        self.idx = Idx::Parent(idx);

        self.opened.insert(idx);
    }

    fn insert_request(&mut self, collection_idx: usize, idx: usize, req: RequestModel) {
        if let Some(coll) = self.collections.get_mut(collection_idx) {
            coll.requests.insert(idx, req);
        }
    }

    fn remove_collection(&mut self, collection_idx: usize) -> Option<CollectionsModel> {
        if collection_idx < self.collections.len() {
            // TODO: check for collections_idx != self.idx
            self.opened.remove(&collection_idx);

            if collection_idx == self.collections.len() - 1 {
                if collection_idx == 0 {
                    // We are removing the last remaining element in the list
                    self.idx = Idx::None;
                } else {
                    // If removing last, we need go back one
                    self.idx = Idx::Parent(collection_idx - 1);
                }
            }

            Some(self.collections.remove(collection_idx))
        } else {
            None
        }
    }

    fn remove_request(&mut self, (i, sub_i): (usize, usize)) -> Option<RequestModel> {
        if let Some(coll) = self.collections.get_mut(i) {
            if sub_i < coll.requests.len() {
                if sub_i == coll.requests.len() - 1 {
                    if sub_i == 0 {
                        // We are removing the last remaining element in the list
                        self.idx = Idx::Parent(i);
                    } else {
                        // If removing last, we need go back one
                        self.idx = Idx::Child(i, sub_i - 1);
                    }
                }

                return Some(coll.requests.remove(sub_i));
            }
        }

        None
    }

    fn get_collection(&self, idx: usize) -> Option<&CollectionsModel> {
        self.collections.get(idx)
    }

    fn get_request(&self, (i, sub_i): (usize, usize)) -> Option<&RequestModel> {
        if let Some(coll) = self.collections.get(i) {
            return coll.requests.get(sub_i);
        }
        None
    }

    fn get_collection_mut(&mut self, idx: usize) -> Option<&mut CollectionsModel> {
        self.collections.get_mut(idx)
    }

    fn get_request_mut(&mut self, idx: (usize, usize)) -> Option<&mut RequestModel> {
        self.collections
            .get_mut(idx.0)
            .and_then(|coll| coll.requests.get_mut(idx.1))
    }

    fn edit_request<F: FnMut(&mut RequestModel)>(&mut self, idx: (usize, usize), mut f: F) {
        if let Some(req) = self.get_request_mut(idx) {
            f(req)
        }
    }
}

/// Specific enum index type for only 1 level or nesting
#[derive(Default, Debug, Clone, PartialEq, Eq, Copy)]
pub enum Idx {
    #[default]
    None,
    Parent(usize),
    Child(usize, usize),
}

impl PartialOrd for Idx {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // let ord = match (self, other) {
        //     (Idx::None, Idx::None) => std::cmp::Ordering::Equal,
        //     (Idx::None, Idx::Parent(_)) => std::cmp::Ordering::Less,
        //     (Idx::None, Idx::Child(_, _)) => std::cmp::Ordering::Less,

        //     (Idx::Parent(_), Idx::None) => std::cmp::Ordering::Greater,
        //     (Idx::Parent(i), Idx::Parent(other_i)) => i.cmp(other_i),
        //     (Idx::Parent(i), Idx::Child(other_i, _other_sub_i)) => {
        //         if i == other_i {
        //             std::cmp::Ordering::Less
        //         } else {
        //             i.cmp(other_i)
        //         }
        //     }

        //     (Idx::Child(_, _), Idx::None) => std::cmp::Ordering::Greater,
        //     (Idx::Child(i, _), Idx::Parent(other_i)) => {
        //         if i == other_i {
        //             std::cmp::Ordering::Greater
        //         } else {
        //             i.cmp(other_i)
        //         }
        //     }
        //     (Idx::Child(i, sub_i), Idx::Child(other_i, other_sub_i)) => {
        //         if i == other_i {
        //             sub_i.cmp(other_sub_i)
        //         } else {
        //             i.cmp(other_i)
        //         }
        //     }
        // };

        Some(self.cmp(other))
    }
}

impl Ord for Idx {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Idx {
    pub fn is_none(&self) -> bool {
        matches!(self, Idx::None)
    }

    // pub fn is_parent(&self) -> bool {
    //     matches!(self, Idx::Parent(_))
    // }

    // pub fn is_child(&self) -> bool {
    //     matches!(self, Idx::Child(_, _))
    // }

    pub fn parent_idx(&self) -> usize {
        match self {
            Idx::Parent(i) | Idx::Child(i, _) => *i,
            Idx::None => unreachable!(),
        }
    }

    pub fn child_idx(&self) -> (usize, usize) {
        if let Idx::Child(i, sub_i) = self {
            (*i, *sub_i)
        } else {
            unreachable!()
        }
    }
}
