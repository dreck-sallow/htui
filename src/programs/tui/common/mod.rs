pub mod action_history;
pub mod component;

pub mod input;

/// Common utils for work with lists
pub mod list_utils {
    /// Go to next item index on the list
    pub fn next(current: Option<usize>, len: usize) -> Option<usize> {
        match current {
            Some(idx) => {
                // One fore the last item: so next to last
                if idx < len.saturating_sub(1) {
                    return Some(idx + 1);
                }
            }
            None => {
                if len > 0 {
                    return Some(0);
                }
            }
        }

        current
    }

    /// Go to next previous index on the list
    pub fn prev(current: Option<usize>) -> Option<usize> {
        if let Some(idx) = current {
            if idx > 0 {
                return Some(idx - 1);
            }
        }
        current
    }
}
