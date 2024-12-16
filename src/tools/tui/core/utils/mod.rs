pub fn next_index<T>(vec: &Vec<T>, index: usize) -> Option<usize> {
    if index < (vec.len() - 1) {
        return Some(index + 1);
    }
    None
}

pub fn prev_index<T>(vec: &Vec<T>, index: usize) -> Option<usize> {
    if index == 0 {
        return None;
    }
    return Some(index - 1);
}
