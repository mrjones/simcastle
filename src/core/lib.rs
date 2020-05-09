pub mod character;
pub mod workforce;

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
        let mut production: i32 = 0;
        for (id, job) in self.workforce.assignments() {
            if *job == workforce::Job::FARMER {
                let c = self.workforce.character_with_id(id.clone()).expect("food_production::character_with_id");
                production += 10;
                if c.get_trait(character::Trait::INTELLIGENCE) > 55 {
                    production += 1;
                }
            }
        }

        return production;
    }
}
