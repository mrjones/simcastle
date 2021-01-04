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
    assignments: std::collections::HashMap<character::CharacterId, Job>,
}

impl Workforce {
    pub fn new(initial_ids: std::collections::HashSet<character::CharacterId>) -> Workforce {
        return Workforce {
            teams: maplit::hashmap!{
                Job::BUILDER => team::Team::new(),
                Job::FARMER => team::Team::new(),
            },
            unassigned: team::Team::new_with_ids(initial_ids),
            assignments: maplit::hashmap!{},
        }
    }

    pub fn advance_turn(&mut self) {
        for (&_, ref mut team) in &mut self.teams {
            team.advance_turn();
        }
        self.unassigned.advance_turn();
    }

    pub fn add_unassigned(&mut self, char_id: character::CharacterId) {
        assert!(!self.assignments.contains_key(&char_id));
        self.unassigned.add(&char_id);
    }

    pub fn team(&self, job: &Job) -> anyhow::Result<&team::Team> {
        return self.teams.get(job).ok_or_else(|| anyhow!("Unknown team: {:?}", job));
    }

    pub fn teams(&self) -> impl std::iter::Iterator<Item=(&Job, &team::Team)> {
        return self.teams.iter();
    }

    pub fn mut_team(&mut self, job: &Job) -> anyhow::Result<&mut team::Team> {
        return self.teams.get_mut(job).ok_or_else(|| anyhow!("Unknown team: {:?}", job));
    }

    fn unset_old_assignment(&mut self, char_id: character::CharacterId, job: Job) {
        match self.assignments.get(&char_id).map(|jobref| *jobref) {
            Some(old_job) => {
                let old_team = self.mut_team(&old_job).expect(&format!("old_job doesn't have team {:?}", job));
                assert!(old_team.contains(&char_id));
                old_team.remove(&char_id);
                self.assignments.remove(&char_id);
            },
            None => {
                assert!(self.unassigned.contains(&char_id));
                self.unassigned.remove(&char_id);
            }
        }
    }

    pub fn assign(&mut self, char_id: character::CharacterId, job: Job) -> anyhow::Result<()> {
        self.unset_old_assignment(char_id, job);
        self.mut_team(&job)?.add(&char_id);
        self.assignments.insert(char_id, job);
        return Ok(());
    }

    pub fn unassign(&mut self, char_id: character::CharacterId, job: Job) -> anyhow::Result<()> {
        self.unset_old_assignment(char_id, job);
        self.unassigned.add(&char_id);
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
