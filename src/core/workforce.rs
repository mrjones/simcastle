use super::character;
use super::team;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum Job {
    BUILDER,
    FARMER,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Workforce {
    unassigned: team::Team,

    teams: std::collections::HashMap<Job, team::Team>,
}

impl Workforce {
    pub fn new(initial_ids: std::collections::HashSet<character::CharacterId>) -> Workforce {
        return Workforce {
            teams: maplit::hashmap!{
                Job::BUILDER => team::Team::new(),
                Job::FARMER => team::Team::new(),
            },
            unassigned: team::Team::new_with_ids(initial_ids),
        }
    }

    pub fn advance_turn(&mut self) {
        for (&_, ref mut team) in &mut self.teams {
            team.advance_turn();
        }
        self.unassigned.advance_turn();
    }

    pub fn add_unassigned(&mut self, char_id: character::CharacterId) {
        self.unassigned.add(&char_id);
    }

    pub fn team(&self, job: &Job) -> anyhow::Result<&team::Team> {
        return self.teams.get(job).ok_or_else(|| anyhow!("Unknown team: {:?}", job));
    }

    pub fn mut_team(&mut self, job: &Job) -> anyhow::Result<&mut team::Team> {
        return self.teams.get_mut(job).ok_or_else(|| anyhow!("Unknown team: {:?}", job));
    }

    pub fn assign(&mut self, char_id: character::CharacterId, job: Job) -> anyhow::Result<()> {
        if !self.unassigned.contains(&char_id) {
            return Err(anyhow!("{} wasn't in {:?}", char_id, self.unassigned.members()));
        }

        self.unassigned.remove(&char_id);
        self.mut_team(&job)?.add(&char_id);
        return Ok(());
    }

    pub fn farmers(&self) -> &team::Team {
        return self.team(&Job::FARMER).expect("FARMERS");
    }

    pub fn builders(&self) -> &team::Team {
        return self.team(&Job::BUILDER).expect("BUILDERS");
    }

    pub fn unassigned(&self) -> &team::Team {
        return &self.unassigned;
    }
}
