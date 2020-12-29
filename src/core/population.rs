use super::character;

use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct Population {
    characters: Vec<character::Character>,

    // TODO(mrjones): Is this the right place for this to live?
    rapport_tracker: RapportTracker,
}

impl Population {
    pub fn new(characters: Vec<character::Character>) -> Population {
        return Population{
            characters: characters,
            rapport_tracker: RapportTracker::new(),
        };
    }

    pub fn characters(&self) -> &Vec<character::Character> {
        return &self.characters;
    }

    pub fn character_with_id(&self, id: character::CharacterId) -> Option<&character::Character> {
        return self.characters.iter().find(|c| c.id() == id);
    }

    pub fn add(&mut self, c: character::Character) {
        self.characters.push(c);
    }

    pub fn rapport_tracker(&self) -> &RapportTracker {
        return &self.rapport_tracker;
    }

    pub fn mut_rapport_tracker(&mut self) -> &mut RapportTracker {
        return &mut self.rapport_tracker;
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct RapportTracker {
    turns_on_same_team: std::collections::HashMap<String, i32>,
}

impl RapportTracker {
    pub fn new() -> RapportTracker {
        return RapportTracker {
            turns_on_same_team: std::collections::HashMap::new(),
        }
    }

    pub fn inc_turns_on_same_team(&mut self, a: &character::CharacterId, b: &character::CharacterId) {
        *self.turns_on_same_team.entry(RapportTracker::pair_key(&a, &b)).or_insert(0) += 1;
    }

    pub fn turns_on_same_team(&self, a: &character::CharacterId, b: &character::CharacterId) -> i32 {
        return self.turns_on_same_team.get(&RapportTracker::pair_key(&a, &b)).map(|x| x.clone()).unwrap_or(0);
    }

    fn pair_key(a: &character::CharacterId, b: &character::CharacterId) -> String {
        return format!("{}:{}", std::cmp::min(a, b), std::cmp::max(a, b));
    }
}
