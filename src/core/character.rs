extern crate rand;

use rand::Rng;

pub struct Character {
    name: String,
}

impl Character {
    pub fn new_with_random_name() -> Character {
        return Character{name: random_name()};
    }

    pub fn name(&self) -> &str {
        return &self.name;
    }
}

fn random_name() -> String {
    let mut rng = rand::thread_rng();
    let names = vec![
        "Alpha", "Bravo", "Charlie", "Delta", "Echo", "Foxtrot", "Golf",
        "Hotel", "India", "Juliet", "Kilo", "Lima", "Mike", "November",
        "Oscar", "Papa", "Quebec", "Romeo", "Sierra", "Tango", "Uniform",
        "Victor", "Whiskey", "X-Ray", "Yankee", "Zulu",
    ];
    return names[rng.gen_range(0, names.len())].to_string();
}
