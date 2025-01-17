/// Enum used for representing the state for
/// diferent item
pub enum NestedListITemState<S, M> {
    Single(S),
    Multiple(M),
}

/// Type used for normalize the two diferent branches
/// for each item
pub enum NestedListItem<S, M> {
    Single(SingleItem<S>),
    Multiple(GroupItem<M, S>),
}

impl<S, M> NestedListItem<S, M> {
    pub fn single(inner: S) -> Self {
        Self::Single(SingleItem(inner))
    }

    pub fn multiple(inner: M) -> Self {
        Self::Multiple(GroupItem::new(inner))
    }
}

pub struct SingleItem<T>(pub T);

impl<T: Clone> Clone for SingleItem<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

pub struct GroupItem<G, S> {
    inner: G,
    items: Vec<SingleItem<S>>,
}

impl<T: Clone, S: Clone> Clone for GroupItem<T, S> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            items: self.items.clone(),
        }
    }
}

impl<T, S> GroupItem<T, S> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            items: Vec::new(),
        }
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub fn sub_items(&self) -> &Vec<SingleItem<S>> {
        &self.items
    }

    pub fn with_children(mut self, items: Vec<SingleItem<S>>) -> Self {
        self.items = items;
        self
    }

    pub fn has_children(&self) -> bool {
        !self.items.is_empty()
    }

    pub fn count_children(&self) -> usize {
        self.items.len()
    }

    pub fn remove_child(&mut self, index: usize) {
        if index <= (self.items.len().saturating_sub(1)) {
            self.items.remove(index);
        }
    }

    pub fn child(&self, idx: usize) -> &SingleItem<S> {
        &self.items[idx]
    }

    pub fn add_child(&mut self, item: SingleItem<S>) {
        self.items.push(item);
    }
}
