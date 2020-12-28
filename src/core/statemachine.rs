pub struct StateMachine<S, D> {
    state: S,
    apply_fn: Box<dyn Fn(&mut S, &D)>,
}

impl <S, D> StateMachine<S, D> {
    pub fn new(initial_state: S, apply_fn: Box<dyn Fn(&mut S, &D)>) -> StateMachine<S, D> {
        return StateMachine{state: initial_state, apply_fn: apply_fn};
    }

    pub fn apply(&mut self, delta: &D) {
        (*self.apply_fn)(&mut self.state, delta);
    }

    pub fn state(&self) -> &S {
        return &self.state;
    }

    pub fn unsafe_state(&mut self) -> &mut S {
        return &mut self.state;
    }
}
