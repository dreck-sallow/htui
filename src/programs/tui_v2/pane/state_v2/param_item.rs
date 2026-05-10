use super::table::TableRow;

#[derive(Clone)]
pub struct ParamItem {
    pub enable: bool,
    pub key: String,
    pub value: String,
}

impl ParamItem {
    pub fn new(key: String, value: String) -> Self {
        Self {
            enable: true,
            key,
            value,
        }
    }

    pub fn empty() -> Self {
        Self {
            enable: true,
            key: "".into(),
            value: "".into(),
        }
    }

    pub fn toggle_enable(&mut self) {
        self.enable = !self.enable;
    }
}

impl TableRow for ParamItem {
    fn next_cell(&self, idx: Option<usize>) -> Option<usize> {
        self.next_with_limit(idx, 2)
    }
}
