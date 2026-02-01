use std::collections::VecDeque;

pub enum PaneActionEffect {
    ChangeIdx,
}

pub struct EffectsCollector {
    list: VecDeque<PaneActionEffect>,
}

impl EffectsCollector {
    pub fn new() -> Self {
        Self {
            list: VecDeque::new(),
        }
    }

    pub fn add(&mut self, effect: PaneActionEffect) {
        self.list.push_back(effect);
    }

    pub fn next(&mut self) -> Option<PaneActionEffect> {
        self.list.pop_front()
    }
}
