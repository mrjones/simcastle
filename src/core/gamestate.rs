use super::character;
use super::workforce;

pub struct GameState {
    workforce: workforce::Workforce,
}

pub struct GameSpec {
    pub initial_characters: i32,
}

impl GameState {
    pub fn init(spec: GameSpec) -> GameState {
        let character_gen = character::CharacterFactory::new();
        return GameState{
            workforce: workforce::Workforce::new(
                (0..spec.initial_characters).map(|_| character_gen.new_character()).collect::<Vec<character::Character>>()),
        }

    }

    pub fn workforce(&self) -> &workforce::Workforce {
        return &self.workforce;
    }

    pub fn mut_workforce(&mut self) -> &mut workforce::Workforce {
        return &mut self.workforce;
    }

}
