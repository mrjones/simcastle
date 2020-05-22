use super::character;

pub struct Population {
    characters: Vec<character::Character>,
}

impl Population {
    pub fn new(characters: Vec<character::Character>) -> Population {
        return Population{
            characters: characters,
        };
    }

    pub fn characters(&self) -> &Vec<character::Character> {
        return &self.characters;
    }

    pub fn character_with_id(&self, id: character::CharacterId) -> Option<&character::Character> {
        return self.characters.iter().find(|c| c.id() == id);
    }

    pub fn add(&mut self, c: character::Character) {
        self.characters.push(c);
    }
}
