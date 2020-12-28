use super::castle;
use super::character;
use super::economy;
use super::population;
use super::types;
use super::workforce;

use rand::Rng;
use serde::{Deserialize, Serialize};

pub struct GameSpec {
    pub initial_potential_characters: usize,
    pub initial_characters: usize,
}

struct GameStateMachine {
    workforce: workforce::Workforce,
    population: population::Population,
    castle: castle::Castle,
    pub turn: i32,

    pub food: types::Millis,
    character_gen: character::CharacterFactory,
}


#[derive(Deserialize, Serialize)]
enum PersisterEntry {
    Command{c: Command},
    InternalMutation{m: InternalMutation},
}


// TODO list:
// - handle errors (results)
// - internal mutations too
// - open and write to file
// - move to separate module
// - replay (and test)
// - checkpointing
struct Saver<'a> {
    sink: &'a mut dyn std::io::Write,
}

impl <'a> Saver<'a> {
    pub fn append_command(&mut self, command: Command) {
        let e = PersisterEntry::Command{c: command};
        let as_json = serde_json::to_string(&e).expect("XXX");
        self.sink.write(as_json.as_bytes()).expect("xxx");
        self.sink.write("\n".as_bytes()).expect("xxx");
    }
}

struct Loader<BRT: std::io::BufRead> {
//    reader: &'a mut BRT,
    lines: std::io::Lines<BRT>,
}

impl <BRT: std::io::BufRead> Loader<BRT> {
    pub fn replay(self) -> impl std::iter::Iterator<Item=PersisterEntry> {
        return self.lines.map(|line| {
            let line = line.unwrap();
            return serde_json::from_str(&line)
                .expect(&format!("parsing: {}", line));
        });
    }
}

#[cfg(test)]
mod persister_tests {
    use super::Saver;
    use super::Loader;

    #[test]
    fn simple() {
        let mut data: Vec<u8> = vec![];

        let cmd1 = super::Command::AddCharacter{
            character: super::character::Character::new_random(super::character::CharacterId(99)),
        };

        let cmd2 = super::Command::AssignToTeam{
            cid: crate::character::CharacterId(99),
            job: crate::workforce::Job::FARMER,
        };

        {
            let mut saver = Saver{
                sink: &mut data,
            };

            saver.append_command(cmd1);
            saver.append_command(cmd2);
        }

        let reader = std::io::BufReader::new(data.as_slice());
        use std::io::BufRead;
        let loader = Loader{
            lines: reader.lines(),
        };

        let results: Vec<super::PersisterEntry> = loader.replay().collect();

        assert_eq!(2, results.len(), "Expected one log");
        match &results[0] {
            &super::PersisterEntry::Command{ref c} => {
                match c {
                    &super::Command::AddCharacter{ref character} => {
                        assert_eq!(character.id(), crate::character::CharacterId(99));
//                        assert_eq!(job, crate::workforce::Job::FARMER);
                    },
                    _ => assert!(false, "Wrong Command type"),
                }
            },
            _ => assert!(false, "Wrong PersisterEntry type"),
        }
    }
}

impl GameStateMachine {
    fn execute_command(&mut self, command: &Command) {
        match command {
            Command::AssignToTeam{cid, job} => {
                self.workforce.assign(*cid, *job);
            },
            Command::AddCharacter{character} => {
                let cid = character.id();
                self.population.add(character.clone());
                self.workforce.add_unassigned(cid);
            },
        }
    }

    fn execute_mutation(&mut self, mutation: &InternalMutation) {
        match mutation {
            InternalMutation::AdvanceTurn{turn} => {
                self.turn = *turn;
            }
        }
    }
}

pub enum Prompt {
    AsylumSeeker(character::Character),
}

pub struct GameState {
    machine: GameStateMachine,
}


#[derive(Serialize, Deserialize, Debug)]
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
            machine: GameStateMachine{
                workforce: workforce::Workforce::new(
                    initial_characters.iter().map(character::Character::id).collect()),
                population: population::Population::new(initial_characters),
                castle: castle::Castle::init(&spec),
                turn: 0,
                food: types::Millis::from_i32(2 * spec.initial_characters as i32),
                character_gen: character_gen,
            },
        };
    }

    pub fn execute_command(&mut self, command: &Command) {
        self.machine.execute_command(command);
    }

    // TODO(mrjones): Make GameState immutable, and make this return a copy?
    pub fn advance_turn(&mut self) -> Vec<Prompt> {
        // TODO(mrjones): Starvation
        self.machine.food = std::cmp::min(
            self.machine.castle.food_infrastructure.food_storage,
            self.machine.food + self.food_delta());

        self.machine.workforce.advance_turn();

        for (c1, c2) in self.machine.workforce.farmers().member_pairs() {
            self.machine.population.mut_rapport_tracker().inc_turns_on_same_team(&c1, &c2);
        }

        // TODO: Need to decide what explicitly gets written down, and what gets
        // recomputed by the execute_mutation framework...
        self.machine.execute_mutation(&InternalMutation::AdvanceTurn{
            turn: self.machine.turn + 1,
        });

        let mut prompts = vec![];
        if rand::thread_rng().gen_bool(0.1) {
            prompts.push(Prompt::AsylumSeeker(
                self.machine.character_gen.new_character()));
        }
        return prompts;
    }

    pub fn food_economy(&self) -> economy::FoodEconomy {
        return economy::food(self.machine.workforce.farmers(), &self.machine.castle.food_infrastructure, &self.machine.population);
    }

    pub fn food_delta(&self) -> types::Millis {
        let food_economy = self.food_economy();
        return types::Millis::from_f32(food_economy.production.eval()) - food_economy.consumed_per_turn;
    }

    pub fn population(&self) -> &population::Population {
        return &self.machine.population;
    }

    pub fn workforce(&self) -> &workforce::Workforce {
        return &self.machine.workforce;
    }

    pub fn castle(&self) -> &castle::Castle {
        return &self.machine.castle;
    }

    pub fn turn(&self) -> i32 {
        return self.machine.turn;
    }

    pub fn food(&self) -> types::Millis {
        return self.machine.food;
    }

}
