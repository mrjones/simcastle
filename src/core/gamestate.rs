use super::castle;
use super::character;
use super::economy;
use super::workforce;

pub struct GameSpec {
    pub initial_characters: i32,
}

pub struct GameState {
    workforce: workforce::Workforce,
    castle: castle::Castle,
    pub turn: i32,

    pub food: i32,
}


impl GameState {
    pub fn init(spec: GameSpec) -> GameState {
        let character_gen = character::CharacterFactory::new();
        return GameState{
            workforce: workforce::Workforce::new(
                (0..spec.initial_characters).map(|_| character_gen.new_character()).collect::<Vec<character::Character>>()),
            castle: castle::Castle::new(),
            turn: 0,
            food: 2 * spec.initial_characters,
        }
    }

    // TODO(mrjones): Make GameState immutable, and make this return a copy?
    pub fn advance_turn(&mut self) {
        self.turn = self.turn + 1;

        // TODO(mrjones): Starvation
        self.food = std::cmp::min(
            self.castle.food_storage,
            self.food + self.food_delta());
    }

    pub fn food_delta(&self) -> i32 {
        return economy::food_production(&self.workforce) - self.workforce.population().len() as i32;
    }

    pub fn workforce(&self) -> &workforce::Workforce {
        return &self.workforce;
    }

    pub fn mut_workforce(&mut self) -> &mut workforce::Workforce {
        return &mut self.workforce;
    }

    pub fn castle(&self) -> &castle::Castle {
        return &self.castle;
    }
}
