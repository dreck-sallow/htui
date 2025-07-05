use super::{mutations::PaneStateMutation, PaneState};

pub struct MutationsHistory<'a> {
    stack: Vec<Box<dyn PaneStateMutation + 'a>>,
    cursor: Option<usize>,
}

impl<'a> MutationsHistory<'a> {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            cursor: None,
        }
    }

    pub fn collector(&self) -> MutationCollector<'a> {
        MutationCollector::new()
    }

    pub fn apply_from_collector(
        &mut self,
        collector: MutationCollector<'a>,
        state: &mut PaneState,
    ) {
        for mutation in collector.mutations {
            self.apply(mutation, state);
        }
    }

    fn apply(&mut self, mut mutation: Box<dyn PaneStateMutation + 'a>, state: &mut PaneState) {
        mutation.apply(state);
        self.stack.push(mutation);

        self.cursor = match self.cursor {
            Some(i) => Some(i + 1),
            None => Some(0),
        };
    }

    pub fn go_forward(&mut self, state: &mut PaneState) {
        match self.cursor {
            Some(cursor) => {
                if cursor < (self.stack.len() - 1) {
                    self.stack[cursor + 1].apply(state);
                    self.cursor = Some(cursor + 1);
                }
            }
            None => {
                if !self.stack.is_empty() {
                    self.stack[0].apply(state);
                    self.cursor = Some(0);
                }
            }
        }
    }

    pub fn go_back(&mut self, state: &mut PaneState) {
        if let Some(cursor) = self.cursor {
            if cursor > 0 {
                self.stack[cursor - 1].apply(state);
                self.cursor = Some(cursor - 1);
            }
        }
    }
}

pub struct MutationCollector<'a> {
    mutations: Vec<Box<dyn PaneStateMutation + 'a>>,
}

impl<'a> MutationCollector<'a> {
    fn new() -> Self {
        Self {
            mutations: Vec::new(),
        }
    }

    pub fn add<M: PaneStateMutation + 'a>(&mut self, mutation: M) {
        self.mutations.push(Box::new(mutation));
    }

    pub fn has_content(&self) -> bool {
        !self.mutations.is_empty()
    }
}
