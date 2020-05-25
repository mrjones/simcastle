use super::castle;
use super::character;
use super::economy;
use super::population;
use super::types;
use super::workforce;

use rand::Rng;

pub struct GameSpec {
    pub initial_potential_characters: usize,
    pub initial_characters: usize,
}

pub struct GameState {
    workforce: workforce::Workforce,
    population: population::Population,
    castle: castle::Castle,
    pub turn: i32,

    pub food: types::Millis,
    character_gen: character::CharacterFactory,
}

pub enum Prompt {
    AsylumSeeker(character::Character),
}

impl GameState {
    pub fn init(spec: GameSpec, initial_characters: Vec<character::Character>, character_gen: character::CharacterFactory) -> GameState {
        assert_eq!(initial_characters.len(), spec.initial_characters as usize,
                   "Please pick {} initial characters ({} selected)",
                   spec.initial_characters, initial_characters.len());
        return GameState{
            workforce: workforce::Workforce::new(
                initial_characters.iter().map(character::Character::id).collect()),
            population: population::Population::new(initial_characters),
            castle: castle::Castle::init(&spec),
            turn: 0,
            food: types::Millis::from_i32(2 * spec.initial_characters as i32),
            character_gen: character_gen,
        }
    }

    // TODO(mrjones): Make GameState immutable, and make this return a copy?
    pub fn advance_turn(&mut self) -> Vec<Prompt> {
        self.turn = self.turn + 1;

        // TODO(mrjones): Starvation
        self.food = std::cmp::min(
            self.castle.food_infrastructure.food_storage,
            self.food + self.food_delta());

        self.workforce.advance_turn();

        for (c1, c2) in self.workforce.farmers().member_pairs() {
            self.population.mut_rapport_tracker().inc_turns_on_same_team(&c1, &c2);
        }

        let mut prompts = vec![];
        if rand::thread_rng().gen_bool(0.1) {
            prompts.push(Prompt::AsylumSeeker(
                self.character_gen.new_character()));
        }
        return prompts;
    }

    pub fn food_economy(&self) -> economy::FoodEconomy {
        return economy::food(self.workforce.farmers(), &self.castle.food_infrastructure, &self.population);
    }

    pub fn food_delta(&self) -> types::Millis {
        let food_economy = self.food_economy();
        return food_economy.produced_per_turn - food_economy.consumed_per_turn;
    }

    pub fn population(&self) -> &population::Population {
        return &self.population;
    }

    pub fn mut_population(&mut self) -> &mut population::Population {
        return &mut self.population;
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

    pub fn accept_asylum_seeker(&mut self, c: character::Character) {
        let cid = c.id();
        self.population.add(c);
        self.workforce.add_unassigned(cid);
    }
}
