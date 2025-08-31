#[derive(Clone, Copy)]
pub enum CursorType {
    Block,
    Bar,
    Underline,
}

/// Input text used for only 1 line of text
pub struct Input {
    inner: String,
    cursor_pos: usize,
    selection_start: Option<usize>,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            inner: String::new(),
            cursor_pos: 0,
            selection_start: None,
        }
    }
}

impl Input {
    pub fn txt(&self) -> &str {
        &self.inner
    }

    pub fn cursor(&self) -> usize {
        self.cursor_pos
    }

    pub fn selection_range(&self) -> Option<(usize, usize)> {
        self.selection_start
            .map(|start| (start.min(self.cursor_pos), start.max(self.cursor_pos)))
    }

    pub fn has_range(&self) -> bool {
        self.selection_start
            .map_or(false, |start| start != self.cursor_pos)
    }

    pub fn delete_range(&mut self) {
        if let Some((start, end)) = self.selection_range() {
            let end = end.min(self.inner.chars().count() - 1);
            for _ in start..(end + 1) {
                // println!("i: {}", i);
                self.inner.remove(start);
            }
            self.cursor_pos = start;
        }
        self.selection_start = None;
    }

    pub fn clear(&mut self) {
        self.inner.clear();
        self.cursor_pos = 0;
        self.selection_start = None;
    }

    pub fn insert_char(&mut self, ch: char) {
        let byte_idx = self
            .inner
            .char_indices()
            .nth(self.cursor_pos)
            .map_or(self.inner.len(), |(idx, _)| idx);
        self.inner.insert(byte_idx, ch);
        self.cursor_pos += 1;
    }

    pub fn delete_char(&mut self) {
        if self.cursor_pos == 0 {
            return;
        }

        let (byte_idx, _) = self.inner.char_indices().nth(self.cursor_pos - 1).unwrap();

        self.inner.remove(byte_idx);
        self.cursor_pos -= 1;
    }

    pub fn delete_next_char(&mut self) {
        if self.inner.is_empty() {
            return;
        }

        if let Some((byte_idx, _)) = self.inner.char_indices().nth(self.cursor_pos) {
            self.inner.remove(byte_idx);
        }
    }

    pub fn forward_cursor(&mut self, idx: usize) {
        let length = self.inner.chars().count();

        if self.cursor_pos + idx > (length - 1) {
            self.cursor_pos = length;
        } else {
            self.cursor_pos += idx;
        }
    }

    pub fn backward_cursor(&mut self, idx: usize) {
        if idx > self.cursor_pos {
            self.cursor_pos = 0;
        } else {
            self.cursor_pos -= idx;
        }
    }

    pub fn next_cursor_until<F: Fn(char, Option<char>) -> bool>(&mut self, f: F) {
        let mut i = self.cursor_pos + 1;
        // TODO: skip cursor pos
        let chars: Vec<char> = self.inner.chars().collect();

        while let Some(ch) = chars.get(i) {
            if f(*ch, chars.get(i + 1).cloned()) {
                self.cursor_pos = i;
                break;
            }
            i += 1;
        }
    }

    pub fn back_cursor_until<F: Fn(Option<char>, char) -> bool>(&mut self, f: F) {
        if self.cursor_pos == 0 {
            return;
        }

        let mut i = self.cursor_pos - 1;
        // TODO: take until cursor
        let chars: Vec<char> = self.inner.chars().collect();

        while let Some(ch) = chars.get(i) {
            let before_char = if i == 0 { None } else { chars.get(i - 1) };

            if f(before_char.cloned(), *ch) {
                self.cursor_pos = i;
                break;
            }

            if i == 0 {
                break;
            }

            i -= 1;
        }
    }

    pub fn start_selection(&mut self) {
        self.selection_start = Some(self.cursor_pos);
    }

    pub fn clear_selection(&mut self) {
        self.selection_start = None;
    }

    pub fn edit<H: EditHandler>(&mut self, handler: &mut H) -> InputModified {
        handler.edit(self)
    }
}

pub type InputModified = bool;

pub trait EditHandler {
    fn edit(&mut self, input: &mut Input) -> InputModified;
}
