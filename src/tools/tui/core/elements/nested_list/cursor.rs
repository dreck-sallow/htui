pub type IdxType = usize;
pub type Idx = Option<IdxType>;

/// Datatype used for 1 nested level list
#[derive(Default, Clone, Debug, PartialEq, PartialOrd)]
pub struct NestedCursor(Idx, Idx);

impl NestedCursor {
    pub fn new(idx: Idx, sub_idx: Idx) -> Self {
        let sub_idx = if idx.is_none() { None } else { sub_idx };
        Self(idx, sub_idx)
    }

    pub fn empty() -> Self {
        Self::default()
    }

    #[inline]
    pub fn inner(&self) -> (Idx, Idx) {
        (self.0, self.1)
    }

    pub fn idx(&self) -> &Idx {
        &self.0
    }

    pub fn sub_idx(&self) -> &Idx {
        &self.1
    }

    pub fn add_idx(&mut self, n: IdxType) {
        let count = self.0.map_or(0, |i| i.saturating_add(n));
        self.set_idx(Some(count))
    }

    pub fn add_sub_idx(&mut self, n: IdxType) {
        let count = self.1.map_or(0, |i| i.saturating_add(n));
        self.set_sub_idx(Some(count))
    }

    pub fn set_idx(&mut self, idx: Idx) {
        match idx {
            Some(i) => {
                self.0 = Some(i);
                self.1 = None;
            }
            None => {
                self.0 = None;
                self.1 = None;
            }
        }
    }

    pub fn set_sub_idx(&mut self, sub_idx: Idx) {
        if self.0.is_some() {
            self.1 = sub_idx;
        } else {
            self.1 = None
        }
    }
}

impl From<(IdxType, IdxType)> for NestedCursor {
    fn from(value: (IdxType, IdxType)) -> Self {
        Self(Some(value.0), Some(value.1))
    }
}

impl From<IdxType> for NestedCursor {
    fn from(value: IdxType) -> Self {
        Self(Some(value), None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn default_empty_cursor() {
        assert_eq!(NestedCursor::default(), NestedCursor::empty());

        let inner: (Idx, Idx) = (None, None);
        assert_eq!(NestedCursor::default().inner(), inner);
        assert_eq!(NestedCursor::empty().inner(), inner);
    }

    #[test]
    pub fn cursor_values() {
        let mut cursor_1 = NestedCursor::from(1);
        cursor_1.set_idx(None);
        cursor_1.set_sub_idx(Some(2));

        assert_eq!(cursor_1, NestedCursor::empty());

        let mut cursor_2 = NestedCursor::from(1);
        cursor_2.set_idx(Some(4));
        cursor_2.set_sub_idx(None);

        assert_eq!(cursor_2.idx(), &Some(4));
        assert_eq!(cursor_2.sub_idx(), &None);

        cursor_2.set_sub_idx(Some(2));
        assert_eq!(cursor_2.sub_idx(), &Some(2));

        cursor_2.set_idx(None);
        assert_eq!(cursor_2.inner(), (None, None));
    }

    #[test]
    pub fn compare_cursors() {
        let mut cursor_1 = NestedCursor::default();
        cursor_1.set_idx(Some(1));
        cursor_1.set_sub_idx(Some(2));

        assert_eq!(cursor_1, NestedCursor::from((1, 2)));

        let mut cursor_2 = NestedCursor::empty();
        cursor_2.set_idx(Some(2));
        cursor_2.set_sub_idx(Some(0));

        assert_eq!(cursor_2, NestedCursor::from((2, 0)));
    }

    #[test]
    pub fn ordering_cursors() {
        assert!(NestedCursor::empty() < NestedCursor::from(0));
        assert!(NestedCursor::empty() == NestedCursor::new(None, None));
        assert!(NestedCursor::new(Some(1), None) >= NestedCursor::new(Some(0), None));
        assert!(NestedCursor::new(Some(1), Some(2)) < NestedCursor::new(Some(2), None));
        assert!(NestedCursor::new(Some(1), Some(5)) > NestedCursor::new(Some(1), Some(4)));
        assert!(NestedCursor::new(None, None) == NestedCursor::new(None, Some(3)));
    }
}
