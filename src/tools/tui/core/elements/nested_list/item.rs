pub struct NestedListSingle<T>(pub T);

pub struct NestedListMultiple<T, S> {
    inner: T,
    children: Vec<NestedListSingle<S>>,
}

impl<T, S> NestedListMultiple<T, S> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            children: Vec::new(),
        }
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub fn sub_items(&self) -> &Vec<NestedListSingle<S>> {
        &self.children
    }

    pub fn with_children(mut self, children: Vec<NestedListSingle<S>>) -> Self {
        self.children = children;
        self
    }

    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    pub fn count_children(&self) -> usize {
        self.children.len()
    }

    pub fn remove_child(&mut self, index: usize) {
        if index <= (self.children.len().saturating_sub(1)) {
            self.children.remove(index);
        }
    }
}
