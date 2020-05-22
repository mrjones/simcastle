pub mod castle;
pub mod character;
pub mod gamestate;
pub mod population;
pub mod team;
pub mod types;
pub mod workforce;

mod economy;

pub struct Game {
    state: gamestate::GameState,
}

impl Game {
    pub fn new(spec: gamestate::GameSpec) -> Game {
        return Game {
            state: gamestate::GameState::init(spec),
        }
    }

    pub fn state(&self) -> &gamestate::GameState {
        return &self.state;
    }

    pub fn mut_state(&mut self) -> &mut gamestate::GameState {
        return &mut self.state;
    }
}
