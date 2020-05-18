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
}
