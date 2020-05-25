use super::gamestate;

pub struct InitialSetup {
    spec: gamestate::GameSpec,
}

impl InitialSetup {
    pub fn new(spec: gamestate::GameSpec) -> InitialSetup {
        return InitialSetup{
            spec: spec,
        }
    }

    pub fn begin(self) -> gamestate::GameState {
        return gamestate::GameState::init(self.spec);
    }
}
