extern crate rand;
extern crate rand_distr;

use rand::Rng;

pub struct Character {
    name: String,
    intelligence: i32,
}

impl Character {
    pub fn new_random() -> Character {
        return Character{
            name: random_name(),
            intelligence: random_stat(),
        };
    }

    pub fn name(&self) -> &str {
        return &self.name;
    }

    pub fn full_debug_string(&self) -> String {
        return format!("[{}] int:{}", self.name, self.intelligence);
    }
}

fn random_stat() -> i32 {
    let x: f32 = rand::thread_rng().sample(rand_distr::StandardNormal);
    return (10.0 * x) as i32;
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
