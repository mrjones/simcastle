use super::castle;
use super::character;
use super::economy;
use super::population;
use super::statemachine;
use super::types;
use super::workforce;

use anyhow::Context;
use rand::Rng;
use serde::{Deserialize, Serialize};

pub struct GameSpec {
    pub initial_potential_characters: usize,
    pub initial_characters: usize,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct GameStateT {
    pub turn: i32,
    pub food: types::Millis,

    pub population: population::Population,
    pub workforce: workforce::Workforce,
    pub castle: castle::Castle,

    pub next_valid_cid: character::CharacterId,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub enum UserCommand {
    AssignToTeam{cid: character::CharacterId, job: workforce::Job},
    AddCharacter{character: character::Character}
}

#[derive(Clone, Serialize, Deserialize)]
enum MutationT {
    EndTurn,
    SetFood{v: types::Millis},
    UserCommand{cmd: UserCommand},
    UpdateCharacter{character_delta: character::CharacterDelta},
}

fn apply_mutation(state: &mut GameStateT, m: &MutationT) {
    match &m {
        &MutationT::EndTurn => {
            state.turn = state.turn + 1;
            state.workforce.advance_turn();
            for (c1, c2) in state.workforce.farmers().member_pairs() {
                state.population.mut_rapport_tracker().inc_turns_on_same_team(&c1, &c2);
            }
        }
        &MutationT::SetFood{v} => state.food = *v,
        &MutationT::UserCommand{cmd} => apply_user_command(state, cmd),
        &MutationT::UpdateCharacter{character_delta} => {
            let character = state.population.mut_character_with_id(character_delta.id)
                .expect(&format!("no character with id {}", character_delta.id));
            for (&t, &new_v) in &character_delta.changed_trait_values {
                character.mut_trait(t).value = new_v;
            }
        }
    }
}

fn apply_user_command(state: &mut GameStateT, c: &UserCommand) {
    match &c {
        &UserCommand::AssignToTeam{cid, job} => state.workforce.assign(cid.clone(), job.clone()),
        &UserCommand::AddCharacter{character} => {
            if character.id().0 >= state.next_valid_cid.0 {
                state.next_valid_cid = character::CharacterId(character.id().0 + 1);
            }
            state.population.add(character.clone());
            state.workforce.add_unassigned(character.id());
        },
    }
}

pub enum Prompt {
    AsylumSeeker(character::Character),
}

pub struct GameState {
//    save_file: &'svf mut std::fs::File,
    machine: statemachine::PersistentStateMachine<GameStateT, MutationT>,
}

impl GameState {
    pub fn init(spec: GameSpec,
                initial_characters: Vec<character::Character>,
                save_file: std::fs::File) -> anyhow::Result<GameState> {
        assert_eq!(initial_characters.len(), spec.initial_characters as usize,
                   "Please pick {} initial characters ({} selected)",
                   spec.initial_characters, initial_characters.len());

        return Ok(GameState{
            machine: statemachine::PersistentStateMachine::init(
                GameStateT{
                    turn: 0,
                    food: types::Millis::from_i32(2 * spec.initial_characters as i32),
                    workforce: workforce::Workforce::new(
                        initial_characters.iter().map(character::Character::id).collect()),
                    next_valid_cid: initial_characters.iter().fold(
                        character::CharacterId(0),
                        |so_far, candidate| character::CharacterId(std::cmp::max(so_far.0, candidate.id().0 + 1))),
                    population: population::Population::new(initial_characters),
                    castle: castle::Castle::init(&spec),
                },
                Box::new(apply_mutation),
                statemachine::Saver::new(std::rc::Rc::new(std::sync::Mutex::new(save_file))),
            )?,
        });
    }

    fn restore_helper<P: AsRef<std::path::Path> + std::fmt::Debug>(filename: &P) -> anyhow::Result<GameStateT> {
        use std::io::BufRead;

        let restore_file = std::fs::File::open(filename)
            .with_context(|| format!("Opening {:?} for restore", *filename))?;
        let restore_reader = std::io::BufReader::new(restore_file);
        return Ok(statemachine::PersistentStateMachine::recover(
            &mut restore_reader.lines().map(|r| r.expect("error reading line")),
            &apply_mutation)?);
    }

    pub fn restore<P: AsRef<std::path::Path> + std::fmt::Debug>(filename: P) -> anyhow::Result<GameState> {
        let state = GameState::restore_helper(&filename)?;
        let save_file = std::fs::OpenOptions::new().write(true).append(true).open(&filename)
            .with_context(|| format!("Opening {:?} for as save_file", &filename))?;

        return Ok(GameState{
            machine: statemachine::PersistentStateMachine::init(
                state,
                Box::new(apply_mutation),
                statemachine::Saver::new(std::rc::Rc::new(std::sync::Mutex::new(save_file))))?,
        });
    }

    pub fn execute_command(&mut self, command: &UserCommand) -> anyhow::Result<()> {
        return self.machine.apply(&MutationT::UserCommand{cmd: command.clone()});
    }

    // TODO(mrjones): Make GameState immutable, and make this return a copy?
    pub fn advance_turn(&mut self) -> anyhow::Result<Vec<Prompt>> {
        // TODO(mrjones): Starvation
        let food = std::cmp::min(
            self.machine.state().castle.food_infrastructure.food_storage,
            self.machine.state().food + self.food_delta());


        for char_delta in self.machine.state().population.compute_end_of_turn_deltas() {
            self.machine.apply(&MutationT::UpdateCharacter{character_delta: char_delta})?;
        }
        // TODO: Need to decide what explicitly gets written down, and what gets
        // recomputed by the execute_mutation framework...
        self.machine.apply(&MutationT::SetFood{v: food})?;
        self.machine.apply(&MutationT::EndTurn)?;

        let mut prompts = vec![];
        if rand::thread_rng().gen_bool(0.1) {
            prompts.push(Prompt::AsylumSeeker(character::Character::new_random(
                self.machine.state().next_valid_cid)));
        }
        return Ok(prompts);
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
