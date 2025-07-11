use crate::programs::tui::events::EventSender;

use super::PaneState;

/// We will use the sender_event here for the sending request process
/// Wen the process of sending finish, I need notify to events the RE-DRAW
/// Because the process is async, and not is sync with event Draw
// SUGGEST: Not pass the sender, and pass directly to mutation object
pub trait Mutation {
    fn apply(&mut self, state: &mut PaneState, _sender_event: &EventSender);
    fn undo(&mut self, state: &mut PaneState, _sender_event: &EventSender);
}

pub struct MutationsHistory {
    stack: Vec<Box<dyn Mutation + 'static>>,
    cursor: Option<usize>,
}

impl MutationsHistory {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            cursor: None,
        }
    }

    pub fn apply_from_collector(
        &mut self,
        collector: MutationCollector,
        state: &mut PaneState,
        sender: &EventSender,
    ) {
        for mutation in collector.mutations {
            self.apply(mutation, state, sender);
        }
    }

    fn apply(
        &mut self,
        mut mutation: Box<dyn Mutation + 'static>,
        state: &mut PaneState,
        sender_event: &EventSender,
    ) {
        mutation.apply(state, sender_event);
        self.stack.push(mutation);

        self.cursor = match self.cursor {
            Some(i) => Some(i + 1),
            None => Some(0),
        };
    }

    pub fn go_forward(&mut self, state: &mut PaneState, sender: &EventSender) {
        match self.cursor {
            Some(cursor) => {
                if cursor < (self.stack.len() - 1) {
                    self.stack[cursor + 1].apply(state, sender);
                    self.cursor = Some(cursor + 1);
                }
            }
            None => {
                if !self.stack.is_empty() {
                    self.stack[0].apply(state, sender);
                    self.cursor = Some(0);
                }
            }
        }
    }

    pub fn go_back(&mut self, state: &mut PaneState, sender: &EventSender) {
        if let Some(cursor) = self.cursor {
            self.stack[cursor].undo(state, sender);

            if cursor > 0 {
                self.cursor = Some(cursor - 1);
            } else {
                self.cursor = None;
            }
        }
    }
}

pub struct MutationCollector {
    mutations: Vec<Box<dyn Mutation + 'static>>,
}

impl MutationCollector {
    pub fn new() -> Self {
        Self {
            mutations: Vec::new(),
        }
    }

    pub fn add<M: Mutation + 'static>(&mut self, mutation: M) {
        self.mutations.push(Box::new(mutation));
    }
}
