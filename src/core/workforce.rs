use super::character;
use super::team;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Job {
    FARMER,
}

pub struct Workforce {
    population: Vec<character::Character>,

    farmers: team::Team,
    unassigned: team::Team,
}

impl Workforce {
    pub fn new(population: Vec<character::Character>) -> Workforce {
        return Workforce {
            population: population,
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

    pub fn population(&self) -> &Vec<character::Character> {
        return &self.population;
    }

    pub fn character_with_id(&self, id: character::CharacterId) -> Option<&character::Character> {
        return self.population.iter().find(|c| c.id() == id);
    }

    pub fn add_to_population(&mut self, c: character::Character) {
        self.population.push(c);
    }

    pub fn farmers(&self) -> &team::Team {
        return &self.farmers;
    }

    pub fn unassigned(&self) -> &team::Team {
        return &self.unassigned;
    }
}
