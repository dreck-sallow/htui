pub enum NestedListItemState<S, G> {
    Single(S),
    Group(G),
}

pub enum NestedListItem<S, G> {
    Single(S),
    Group {
        inner: G,
        items: Vec<NestedListItem<S, G>>,
    },
}

impl<S, G> NestedListItem<S, G> {
    pub fn is_single(&self) -> bool {
        match self {
            NestedListItem::Single(_) => true,
            NestedListItem::Group { .. } => false,
        }
    }

    pub fn is_group(&self) -> bool {
        !self.is_single()
    }

    pub fn inner(&self) -> NestedListItemState<&S, &G> {
        match self {
            NestedListItem::Single(inner) => NestedListItemState::Single(inner),
            NestedListItem::Group { inner, .. } => NestedListItemState::Group(inner),
        }
    }
}
