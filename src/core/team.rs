use super::character;

use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct Team {
    members: std::collections::HashSet<character::CharacterId>,
}

impl Team {
    pub fn new() -> Team {
        return Team{
            members: std::collections::HashSet::new(),
        };
    }

    pub fn new_with_ids(initial_ids: std::collections::HashSet<character::CharacterId>) -> Team {
        return Team{
            members: initial_ids,
        }
    }

    pub fn add(&mut self, id: &character::CharacterId) {
        self.members.insert(id.clone());
    }

    pub fn remove(&mut self, id: &character::CharacterId) {
        assert!(self.members.remove(id));
    }

    pub fn contains(&self, id: &character::CharacterId) -> bool {
        return self.members.contains(id);
    }

    pub fn members(&self) -> &std::collections::HashSet<character::CharacterId> {
        return &self.members;
    }

    pub fn member_pairs(&self) -> Vec<(character::CharacterId, character::CharacterId)>{
        return self.members().iter().combinations(2).map(|v| {
            assert_eq!(2, v.len());
            return (v[0].clone(), v[1].clone());
        }).collect();
    }

    // Scale factor: 0.0 == average, +/- 1.0 per stddev
    pub fn harmony(&self) -> f32 {
        // Factors: turns working together
        // Leadership:
        // Trait matching:
        return 0.0;
    }

    pub fn advance_turn(&self) {

    }
}
