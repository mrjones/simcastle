use super::character;

pub struct Team {
    members: std::collections::HashSet<character::CharacterId>,
}

impl Team {
    pub fn new() -> Team {
        return Team{
            members: std::collections::HashSet::new(),
        };
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
