use super::character;
use super::gamestate;

pub struct InitialSetup {
    spec: gamestate::GameSpec,
    character_gen: character::CharacterFactory,

    pub character_candidates: Vec<character::Character>,
}

impl InitialSetup {
    pub fn new(spec: gamestate::GameSpec) -> InitialSetup {
        let character_gen = character::CharacterFactory::new();
        let character_candidates = (0..spec.initial_potential_characters).map(|_| character_gen.new_character()).collect::<Vec<character::Character>>();
        return InitialSetup{
            spec: spec,
            character_gen: character_gen,
            character_candidates: character_candidates,
        }
    }

    pub fn spec(&self) -> &gamestate::GameSpec {
        return &self.spec;
    }

    pub fn begin(self, selected_characters: std::collections::HashSet<character::CharacterId>, save_file: std::fs::File) -> gamestate::GameState {
        return gamestate::GameState::init(
            self.spec,
            self.character_candidates.into_iter().filter(|c| selected_characters.contains(&c.id())).collect(),
            self.character_gen,
            save_file);
    }
}
