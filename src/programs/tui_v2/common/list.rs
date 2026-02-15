pub fn page_list<I>(itms: &[I], idx: usize, height: usize) -> Option<(usize, usize)> {
    if itms.is_empty() || height == 0 {
        return None;
    }

    let start = (idx / height) * height;

    Some((start, (start + (height - 1)).min(itms.len() - 1)))
}

pub fn in_page_cursor(idx: usize, height: usize) -> usize {
    idx % height
}

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

pub fn clamp_index(current: Option<usize>, len: usize) -> Option<usize> {
    match current {
        Some(idx) => {
            if len == 0 {
                return None;
            } else if idx >= len {
                return Some(idx - 1);
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

pub fn delete_element<E>(current_idx: Option<usize>, list: &mut Vec<E>) -> Option<usize> {
    match current_idx {
        Some(idx) => {
            list.remove(idx);
            clamp_index(current_idx, list.len())
        }
        None => None,
    }
}
