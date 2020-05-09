extern crate rand;
extern crate rand_distr;
extern crate std;

use rand::Rng;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Trait {
    INTELLIGENCE,
    STRENGTH,
}

pub fn all_traits() -> Vec<Trait> {
    return vec![Trait::INTELLIGENCE, Trait::STRENGTH];
}

pub struct Character {
    name: String,
    traits: std::collections::HashMap<Trait, i32>,
}

impl Character {
    pub fn new_random() -> Character {
        return Character{
            name: random_name(),
            traits: random_traits(),
        };
    }

    pub fn name(&self) -> &str {
        return &self.name;
    }

    pub fn full_debug_string(&self) -> String {
        return format!("[{}] traits:{}",
                       self.name,
                       self.traits.iter().map(|(t, v)| format!("{:?}:{}", t, v)).collect::<Vec<String>>().join(" "));
    }
}

fn random_stat() -> i32 {
    let x: f32 = rand::thread_rng().sample(rand_distr::StandardNormal);
    return (10.0 * x) as i32;
}

fn random_traits() -> std::collections::HashMap<Trait, i32> {
    let mut map: std::collections::HashMap<Trait, i32> = std::collections::HashMap::new();
    for t in all_traits() {
        map.insert(t.clone(), random_stat());
    }
    return map;
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
