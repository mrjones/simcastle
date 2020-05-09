use super::character;

#[derive(Eq, PartialEq)]
pub enum Job {
    FARMER,
}

pub struct Workforce {
    population: Vec<character::Character>,
    assignments: std::collections::HashMap<character::CharacterId, Job>,
}

impl Workforce {
    pub fn new(population: Vec<character::Character>) -> Workforce {
        return Workforce {
            population: population,
            assignments: std::collections::HashMap::new(),
        }
    }

    pub fn assign(&mut self, char_id: character::CharacterId, job: Job) {
        self.assignments.insert(char_id, job);
    }

    pub fn population(&self) -> &Vec<character::Character> {
        return &self.population;
    }

    pub fn character_with_id(&self, id: character::CharacterId) -> Option<&character::Character> {
        return self.population.iter().find(|c| c.id() == id);
    }

    pub fn assignments(&self) -> &std::collections::HashMap<character::CharacterId, Job> {
        return &self.assignments;
    }
}
