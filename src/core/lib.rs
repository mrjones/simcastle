mod character;

pub fn core_logic() {
    println!("Core!")
}


pub struct Game {
    characters: Vec<character::Character>
}

impl Game {
    pub fn new() -> Game {
        return Game{
            characters: (0..3).map(|_| character::Character::new_random()).collect(),
        }
    }

    pub fn characters(&self) -> &Vec<character::Character> {
        return &self.characters;
    }

}
