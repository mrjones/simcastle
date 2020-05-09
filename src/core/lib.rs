pub mod character;
pub mod workforce;

mod economy;

pub struct Game {
    workforce: workforce::Workforce,
}

impl Game {
    pub fn new() -> Game {
        let character_gen = character::CharacterFactory::new();
        return Game{
            workforce: workforce::Workforce::new(
                (0..3).map(|_| character_gen.new_character()).collect::<Vec<character::Character>>()),
        }
    }

    pub fn workforce(&self) -> &workforce::Workforce {
        return &self.workforce;
    }

    pub fn mut_workforce(&mut self) -> &mut workforce::Workforce {
        return &mut self.workforce;
    }

    // XXX
    pub fn food_production(&self) -> i32 {
        return economy::food_production(&self.workforce);
    }
}
