pub trait History<State> {
    type Action: TrackAction<State = State>;

    fn apply(&mut self, action: Self::Action, state: &mut State);
    fn redo(&mut self, state: &mut State);
    fn undo(&mut self, state: &mut State);
    fn clean(&mut self);
}

pub trait TrackAction {
    type State;
    fn apply(&self, state: &mut Self::State) -> Option<Self>
    where
        Self: Sized;
}

pub struct ActionHistory<A> {
    sequence: Vec<A>,
    cursor: Option<usize>,
}

impl<A> ActionHistory<A> {
    pub fn new() -> Self {
        Self {
            sequence: Vec::new(),
            cursor: None,
        }
    }
}

impl<A: TrackAction<State = State>, State> History<State> for ActionHistory<A> {
    type Action = A;

    fn apply(&mut self, action: Self::Action, state: &mut State) {
        let back_action = action.apply(state);

        if let Some(back_action) = back_action {
            if let Some(cursor) = self.cursor {
                let start_i = cursor + 1;
                for _ in start_i..self.sequence.len() {
                    self.sequence.remove(start_i);
                }
            }

            self.sequence.push(back_action);
            self.cursor = Some(self.sequence.len() - 1);
        }
    }

    fn redo(&mut self, state: &mut State) {
        match self.cursor {
            Some(cursor) => {
                if cursor < (self.sequence.len() - 1) {
                    let _action = self.sequence[cursor + 1].apply(state).unwrap();
                    self.sequence[cursor + 1] = _action;

                    self.cursor = Some(cursor + 1);
                }
            }
            None => {
                if !self.sequence.is_empty() {
                    let _action = self.sequence[0].apply(state).unwrap();
                    self.sequence[0] = _action;
                    self.cursor = Some(0);
                }
            }
        }
    }

    fn undo(&mut self, state: &mut State) {
        if let Some(cursor) = self.cursor {
            let _action = self.sequence[cursor].apply(state).unwrap();

            self.sequence[cursor] = _action;
            self.cursor = if cursor == 0 { None } else { Some(cursor - 1) };
        }
    }

    fn clean(&mut self) {
        self.sequence = Vec::new();
        self.cursor = None;
    }
}

#[cfg(test)]
mod test {
    use super::{ActionHistory, History, TrackAction};

    enum CounterAction {
        Add(u8),
        Substract(u8),
        Addone,
        SubstractOne,
        Zero,
    }

    impl TrackAction for CounterAction {
        type State = u8;

        fn apply(&self, state: &mut Self::State) -> Option<Self>
        where
            Self: Sized,
        {
            match self {
                CounterAction::Add(n) => {
                    if let Some(_n) = state.checked_add(*n) {
                        *state = _n;
                        Some(CounterAction::Substract(*n))
                    } else {
                        None
                    }
                }
                CounterAction::Substract(n) => {
                    if let Some(_n) = state.checked_sub(*n) {
                        *state -= *n;
                        Some(CounterAction::Add(*n))
                    } else {
                        None
                    }
                }
                CounterAction::Addone => {
                    if let Some(_n) = state.checked_add(1) {
                        *state += 1;
                        Some(CounterAction::SubstractOne)
                    } else {
                        None
                    }
                }
                CounterAction::SubstractOne => {
                    if let Some(_n) = state.checked_sub(1) {
                        *state -= 1;
                        Some(CounterAction::Addone)
                    } else {
                        None
                    }
                }
                CounterAction::Zero => {
                    let before_state = *state;
                    *state = 0;
                    Some(CounterAction::Add(before_state))
                }
            }
        }
    }

    #[test]
    fn track_actions() {
        let mut state = 3;
        let mut history = ActionHistory::new();

        history.apply(CounterAction::Addone, &mut state);
        history.apply(CounterAction::Addone, &mut state);
        history.apply(CounterAction::Addone, &mut state);
        history.apply(CounterAction::Addone, &mut state);

        assert_eq!(state, 7);
        history.apply(CounterAction::Addone, &mut state);
        assert_eq!(state, 8);

        history.apply(CounterAction::Add(6), &mut state);
        assert_eq!(state, 14);

        history.apply(CounterAction::SubstractOne, &mut state);
        assert_eq!(state, 13);

        history.undo(&mut state);
        history.undo(&mut state);
        assert_eq!(state, 8);

        history.apply(CounterAction::Zero, &mut state);
        assert_eq!(state, 0);

        history.undo(&mut state);
        assert_eq!(state, 8);

        history.redo(&mut state);
        assert_eq!(state, 0);

        history.undo(&mut state);
        history.undo(&mut state);
        history.undo(&mut state);
        history.undo(&mut state);
        history.undo(&mut state);
        assert_eq!(state, 4);

        history.redo(&mut state);
        history.redo(&mut state);
        assert_eq!(state, 6);
    }
}
