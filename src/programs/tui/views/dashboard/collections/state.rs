use std::{collections::HashSet, ops::Not};

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
        let ord = match (self, other) {
            (Idx::None, Idx::None) => std::cmp::Ordering::Equal,
            (Idx::None, Idx::Parent(_)) => std::cmp::Ordering::Less,
            (Idx::None, Idx::Child(_, _)) => std::cmp::Ordering::Less,

            (Idx::Parent(_), Idx::None) => std::cmp::Ordering::Greater,
            (Idx::Parent(i), Idx::Parent(other_i)) => i.cmp(other_i),
            (Idx::Parent(i), Idx::Child(other_i, _other_sub_i)) => {
                if i == other_i {
                    std::cmp::Ordering::Less
                } else {
                    i.cmp(other_i)
                }
            }

            (Idx::Child(_, _), Idx::None) => std::cmp::Ordering::Greater,
            (Idx::Child(i, _), Idx::Parent(other_i)) => {
                if i == other_i {
                    std::cmp::Ordering::Greater
                } else {
                    i.cmp(other_i)
                }
            }
            (Idx::Child(i, sub_i), Idx::Child(other_i, other_sub_i)) => {
                if i == other_i {
                    sub_i.cmp(other_sub_i)
                } else {
                    i.cmp(other_i)
                }
            }
        };

        Some(ord)
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

    pub fn is_parent(&self) -> bool {
        matches!(self, Idx::Parent(_))
    }

    pub fn is_child(&self) -> bool {
        matches!(self, Idx::Child(_, _))
    }
}

pub struct CollectionsState<Identifier> {
    items: Vec<(Identifier, Vec<Identifier>)>,
    idx: Idx,
    opened: HashSet<usize>,
}

impl<Identifier: PartialEq + Eq> CollectionsState<Identifier> {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            idx: Idx::default(),
            opened: HashSet::new(),
        }
    }

    pub fn openeds(&self) -> HashSet<usize> {
        self.opened.clone()
    }

    pub fn idx(&self) -> Idx {
        self.idx.clone()
    }

    pub fn add_collection(&mut self, collection: (Identifier, Vec<Identifier>)) {
        self.items.push(collection);

        if self.idx.is_none() {
            self.idx = Idx::Parent(0);
        }
    }

    pub fn add_request(&mut self, collection: Identifier, request: Identifier) {
        while let Some((coll, requests)) = self.items.iter_mut().next() {
            if *coll == collection {
                requests.push(request);
                break;
            }
        }
    }

    pub fn add_request_on_current(&mut self, request: Identifier) {
        match self.idx {
            Idx::None => {}
            Idx::Parent(i) | Idx::Child(i, _) => self.items[i].1.push(request),
        }
    }

    pub fn next_collection(&mut self) {
        let idx = match self.idx {
            Idx::None => self.items.is_empty().not().then_some(0),
            Idx::Parent(i) | Idx::Child(i, _) => (i < self.items.len() - 1).then_some(i + 1),
        };

        if let Some(i) = idx {
            self.idx = Idx::Parent(i);
        }
    }

    pub fn next_request(&mut self) {
        let child_idx = match self.idx {
            Idx::None => self
                .items
                .first()
                .and_then(|(_, children)| (!children.is_empty()).then_some((0, 0))),
            Idx::Parent(i) => {
                if self.opened.contains(&i) {
                    let (_coll, children) = &self.items[i];
                    (!children.is_empty()).then_some((i, 0))
                } else {
                    None
                }
            }
            Idx::Child(i, sub_i) => {
                let (_coll, children) = &self.items[i];
                (sub_i < children.len() - 1).then_some((i, sub_i + 1))
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
                let previous_idx = self.idx.clone();
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
                .and_then(|prev_i| Some((prev_i, &self.items[prev_i])))
                .and_then(|(prev_i, (_, children))| {
                    (!children.is_empty()).then(|| (prev_i, children.len() - 1))
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
                let previous_idx = self.idx.clone();
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

    pub fn delete_collection(&mut self) {
        // TODO: move the idx on deletion
        match self.idx {
            Idx::Parent(i) => {
                self.items.remove(i);
                self.opened.remove(&i);
            }
            _ => {}
        }
    }

    pub fn delete_request(&mut self) {
        // TODO: move the idx on deletion
        match self.idx {
            Idx::Child(i, sub_i) => {
                self.items[i].1.remove(sub_i);
            }
            _ => {}
        }
    }

    pub fn delete(&mut self) {
        if self.idx.is_parent() {
            self.delete_collection();
        } else if self.idx.is_child() {
            self.delete_request();
        }
    }

    pub fn edit<F: FnMut(&mut Identifier)>(&mut self, mut f: F) {
        match self.idx {
            Idx::None => {}
            Idx::Parent(i) => {
                let id_mut = &mut self.items[i].0;
                f(id_mut);
            }
            Idx::Child(i, sub_i) => {
                let id_mut = &mut self.items[i].1[sub_i];
                f(id_mut);
            }
        }
    }

    pub fn current(&self) -> Option<&Identifier> {
        match self.idx {
            Idx::None => None,
            Idx::Parent(i) => Some(&self.items[i].0),
            Idx::Child(i, sub_i) => Some(&self.items[i].1[sub_i]),
        }
    }
}
