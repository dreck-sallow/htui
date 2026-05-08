#[derive(Default)]
pub enum TableIdx {
    #[default]
    None,
    Row(usize),
    Cell(usize, usize),
}

pub struct TableState<I> {
    idx: TableIdx,
    items: Vec<I>,
}

impl<R> TableState<R> {
    pub fn new() -> Self {
        Self {
            idx: TableIdx::None,
            items: Vec::new(),
        }
    }

    pub fn next_row(&mut self) {
        if self.items.is_empty() {
            return;
        }

        match self.idx {
            TableIdx::None => {
                self.idx = TableIdx::Row(0);
            }
            TableIdx::Row(i) => {
                if self.items.len() - 1 > i {
                    self.idx = TableIdx::Row(i + 1);
                }
            }
            TableIdx::Cell(row, cell) => {
                if self.items.len() - 1 > row {
                    self.idx = TableIdx::Cell(row + 1, cell);
                }
            }
        }
    }

    pub fn prev_row(&mut self) {
        if self.items.is_empty() {
            return;
        }

        match self.idx {
            TableIdx::Row(row) if row > 0 => {
                self.idx = TableIdx::Row(row - 1);
            }
            TableIdx::Cell(row, cell) if row > 0 => {
                self.idx = TableIdx::Cell(row - 1, cell);
            }
            _ => {}
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn add_row(&mut self, row: R) {
        self.items.push(row);

        if self.items.len() == 1 {
            self.idx = TableIdx::Row(1);
        }
    }

    pub fn remove_row(&mut self, idx: usize) -> Option<R> {
        if self.items.is_empty() || (idx > self.items.len() - 1) {
            return None;
        }

        let new_row_idx = |row: usize| -> Option<usize> {
            if idx >= row {
                if idx == 0 {
                    return None;
                } else if idx == self.items.len() - 1 {
                    return Some(row - 1);
                }
                return Some(row);
            }

            Some(row - 1)
        };

        match self.idx {
            TableIdx::Row(row) => {
                self.idx = match new_row_idx(row) {
                    Some(new_row) => TableIdx::Row(new_row),
                    None => TableIdx::None,
                };
            }
            TableIdx::Cell(row, cell) => {
                self.idx = match new_row_idx(row) {
                    Some(new_row) => TableIdx::Cell(new_row, cell),
                    None => TableIdx::None,
                };
            }
            _ => {}
        }

        Some(self.items.remove(idx))
    }

    pub fn current_mut(&mut self) -> Option<&mut R> {
        match self.idx {
            TableIdx::None => None,
            TableIdx::Row(row) | TableIdx::Cell(row, _) => self.items.get_mut(row),
        }
    }
}

impl<R: TableRow> TableState<R> {
    pub fn next_cell(&mut self) {
        if self.items.is_empty() {
            return;
        }

        match self.idx {
            TableIdx::None => {}
            TableIdx::Row(row) => {
                if let Some(cell) = self.items[row].next_cell(None) {
                    self.idx = TableIdx::Cell(row, cell);
                }
            }
            TableIdx::Cell(row, _cell) => {
                if let Some(cell) = self.items[row].next_cell(Some(_cell)) {
                    self.idx = TableIdx::Cell(row, cell);
                }
            }
        }
    }

    pub fn prev_cell(&mut self) {
        if self.items.is_empty() {
            return;
        }

        match self.idx {
            TableIdx::None => {}
            TableIdx::Row(row) => {
                if let Some(cell) = self.items[row].next_cell(None) {
                    self.idx = TableIdx::Cell(row, cell);
                }
            }
            TableIdx::Cell(row, _cell) => {
                if let Some(cell) = self.items[row].prev_cell(Some(_cell)) {
                    self.idx = TableIdx::Cell(row, cell);
                }
            }
        }
    }
}

pub trait TableRow {
    fn next_cell(&self, idx: Option<usize>) -> Option<usize>;
    fn prev_cell(&self, idx: Option<usize>) -> Option<usize> {
        match idx {
            Some(i) if i > 0 => Some(i - 1),
            _ => idx,
        }
    }

    fn next_with_limit(&self, idx: Option<usize>, limit: usize) -> Option<usize> {
        match idx {
            Some(i) if limit > i => Some(i + 1),
            None if limit > 0 => Some(1),
            _ => idx,
        }
    }
}
