use super::castle;
use super::character;
use super::economy;
use super::population;
use super::statemachine;
use super::types;
use super::workforce;

use rand::Rng;
use serde::{Deserialize, Serialize};

pub struct GameSpec {
    pub initial_potential_characters: usize,
    pub initial_characters: usize,
}

pub struct GameStateT {
    pub turn: i32,
    pub food: types::Millis,

    pub population: population::Population,
    pub workforce: workforce::Workforce,
    pub castle: castle::Castle,
}

enum MutationT {
    IncrementTurn,
    SetFood{v: types::Millis},
    UserCommand(Command),
}

fn apply_mutation(state: &mut GameStateT, m: &MutationT) {
    match &m {
        &MutationT::IncrementTurn => state.turn = state.turn + 1,
        &MutationT::SetFood{v} => state.food = *v,
        &MutationT::UserCommand(cmd) => apply_command(state, cmd),
    }
}

fn apply_command(state: &mut GameStateT, c: &Command) {
    unimplemented!();
}

struct GameStateMachine {

}

pub enum Prompt {
    AsylumSeeker(character::Character),
}

pub struct GameState {
    machine: statemachine::StateMachine<GameStateT, MutationT>,

    character_gen: character::CharacterFactory,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Command {
    AssignToTeam{cid: character::CharacterId, job: workforce::Job},
    AddCharacter{character: character::Character}
}

#[derive(Serialize, Deserialize)]
pub enum InternalMutation {
    AdvanceTurn{turn: i32},
}

impl GameState {
    pub fn init(spec: GameSpec, initial_characters: Vec<character::Character>, character_gen: character::CharacterFactory) -> GameState {
        assert_eq!(initial_characters.len(), spec.initial_characters as usize,
                   "Please pick {} initial characters ({} selected)",
                   spec.initial_characters, initial_characters.len());
        return GameState{
            character_gen: character_gen,
            machine: statemachine::StateMachine::new(
                GameStateT{
                    turn: 0,
                    food: types::Millis::from_i32(2 * spec.initial_characters as i32),
                    workforce: workforce::Workforce::new(
                        initial_characters.iter().map(character::Character::id).collect()),
                    population: population::Population::new(initial_characters),
                    castle: castle::Castle::init(&spec),
                }, Box::new(apply_mutation)),
        };
    }

    pub fn execute_command(&mut self, command: &Command) {
        self.machine.apply(&MutationT::UserCommand(command.clone()));
    }

    // TODO(mrjones): Make GameState immutable, and make this return a copy?
    pub fn advance_turn(&mut self) -> Vec<Prompt> {
        // TODO(mrjones): Starvation
        let food = std::cmp::min(
            self.machine.state().castle.food_infrastructure.food_storage,
            self.machine.state().food + self.food_delta());

        self.machine.apply(&MutationT::SetFood{v: food});
        self.machine.unsafe_mutable_state().workforce.advance_turn();

        for (c1, c2) in self.machine.state().workforce.farmers().member_pairs() {
            self.machine.unsafe_mutable_state().population.mut_rapport_tracker().inc_turns_on_same_team(&c1, &c2);
        }

        // TODO: Need to decide what explicitly gets written down, and what gets
        // recomputed by the execute_mutation framework...
        self.machine.apply(&MutationT::IncrementTurn);

        let mut prompts = vec![];
        if rand::thread_rng().gen_bool(0.1) {
            prompts.push(Prompt::AsylumSeeker(
                self.character_gen.new_character()));
        }
        return prompts;
    }

    pub fn food_economy(&self) -> economy::FoodEconomy {
        return economy::food(self.machine.state().workforce.farmers(), &self.machine.state().castle.food_infrastructure, &self.machine.state().population);
    }

    pub fn food_delta(&self) -> types::Millis {
        let food_economy = self.food_economy();
        return types::Millis::from_f32(food_economy.production.eval()) - food_economy.consumed_per_turn;
    }

    pub fn population(&self) -> &population::Population {
        return &self.machine.state().population;
    }

    pub fn workforce(&self) -> &workforce::Workforce {
        return &self.machine.state().workforce;
    }

    pub fn castle(&self) -> &castle::Castle {
        return &self.machine.state().castle;
    }

    pub fn turn(&self) -> i32 {
        return self.machine.state().turn;
    }

    pub fn food(&self) -> types::Millis {
        return self.machine.state().food;
    }

}
