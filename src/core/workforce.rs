use super::character;
use super::team;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum Job {
    BUILDER,
    FARMER,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Workforce {
    farmers: team::Team,
    builders: team::Team,
    unassigned: team::Team,
}

impl Workforce {
    pub fn new(initial_ids: std::collections::HashSet<character::CharacterId>) -> Workforce {
        return Workforce {
            farmers: team::Team::new(),
            builders: team::Team::new(),
            unassigned: team::Team::new_with_ids(initial_ids),
        }
    }

    pub fn advance_turn(&mut self) {
        self.farmers.advance_turn();
        self.unassigned.advance_turn();
    }

    pub fn add_unassigned(&mut self, char_id: character::CharacterId) {
        self.unassigned.add(&char_id);
    }

    pub fn assign(&mut self, char_id: character::CharacterId, job: Job) -> anyhow::Result<()> {
        if !self.unassigned.contains(&char_id) {
            return Err(anyhow!("{} wasn't in {:?}", char_id, self.unassigned.members()));
        }

        self.unassigned.remove(&char_id);

        match job {
            Job::BUILDER => self.builders.add(&char_id),
            Job::FARMER => self.farmers.add(&char_id),
        }

        return Ok(());
    }

    pub fn farmers(&self) -> &team::Team {
        return &self.farmers;
    }

    pub fn builders(&self) -> &team::Team {
        return &self.builders;
    }

    pub fn unassigned(&self) -> &team::Team {
        return &self.unassigned;
    }
}
