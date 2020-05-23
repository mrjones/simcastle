use super::character;
use super::team;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Job {
    FARMER,
}

pub struct Workforce {
    farmers: team::Team,
    unassigned: team::Team,
}

impl Workforce {
    pub fn new(initial_ids: std::collections::HashSet<character::CharacterId>) -> Workforce {
        return Workforce {
            farmers: team::Team::new(),
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

    pub fn assign(&mut self, char_id: character::CharacterId, job: Job) {
        assert!(self.unassigned.contains(&char_id),
                "{} wasn't in {:?}", char_id, self.unassigned.members());
        self.unassigned.remove(&char_id);

        match job {
            Job::FARMER => self.farmers.add(&char_id),
        }
    }

    pub fn farmers(&self) -> &team::Team {
        return &self.farmers;
    }

    pub fn unassigned(&self) -> &team::Team {
        return &self.unassigned;
    }
}
