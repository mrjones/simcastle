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
    pub fn new() -> Workforce {
        return Workforce {
            farmers: team::Team::new(),
            unassigned: team::Team::new(),
        }
    }

    pub fn advance_turn(&mut self) {
        self.farmers.advance_turn();
        self.unassigned.advance_turn();
    }

    pub fn assign(&mut self, char_id: character::CharacterId, job: Job) {
        assert!(self.unassigned.contains(&char_id));
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
