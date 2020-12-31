use super::character;
use super::gamestate;

pub struct InitialSetup {
    spec: gamestate::GameSpec,

    pub character_candidates: Vec<character::Character>,
}

impl InitialSetup {
    pub fn new(spec: gamestate::GameSpec) -> InitialSetup {
        let character_candidates = (0..(spec.initial_potential_characters as i64)).map(|i| character::Character::new_random(character::CharacterId(i))).collect::<Vec<character::Character>>();
        return InitialSetup{
            spec: spec,
            character_candidates: character_candidates,
        }
    }

    pub fn spec(&self) -> &gamestate::GameSpec {
        return &self.spec;
    }

    pub fn begin(self, selected_characters: std::collections::HashSet<character::CharacterId>, save_file: std::fs::File) -> anyhow::Result<gamestate::GameState> {
        return Ok(gamestate::GameState::init(
            self.spec,
            self.character_candidates.into_iter().filter(|c| selected_characters.contains(&c.id())).collect(),
            save_file)?);
    }
}
