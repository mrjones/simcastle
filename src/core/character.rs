extern crate maplit;
extern crate rand;
extern crate rand_distr;
extern crate std;

use rand::Rng;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Deserialize, Serialize, EnumIter)]
pub enum Trait {
    Intelligence,
    Strength,
    WorkEthic,
}

impl Trait {
    pub fn string3(&self) -> &str {
        match self {
            Trait::Intelligence => "INT",
            Trait::Strength => "STR",
            Trait::WorkEthic => "WOR",
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct CharacterId(pub i64);

impl std::fmt::Display for CharacterId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Character {
    id: CharacterId,
    name: String,
    traits: std::collections::HashMap<Trait, i32>,
}

impl Character {
    pub fn new_random(id: CharacterId) -> Character {
        return Character{
            id: id,
            name: random_name(),
            traits: random_traits(),
        };
    }

    pub fn id(&self) -> CharacterId {
        return self.id.clone();
    }

    pub fn name(&self) -> &str {
        return &self.name;
    }

    pub fn get_trait(&self, t: Trait) -> i32 {
        return self.traits.get(&t).expect("unexpected character trait").clone();
    }

    pub fn set_trait(&mut self, t: Trait, v: i32) {
        self.traits.insert(t, v).expect("updated missing trait");
    }

    pub fn full_debug_string(&self) -> String {
        let traits_str = Trait::iter().map(|t| format!("{}:{}", t.string3(), self.get_trait(t.clone()))).collect::<Vec<String>>().join(" ");
        return format!("[{:03}|{:10}] {}", self.id.0, self.name, traits_str);
    }
}

fn random_stat() -> i32 {
    let x: f32 = rand::thread_rng().sample(rand_distr::StandardNormal);
    return 50 + (10.0 * x) as i32;
}

fn random_traits() -> std::collections::HashMap<Trait, i32> {
    let mut map: std::collections::HashMap<Trait, i32> = std::collections::HashMap::new();
    for t in Trait::iter() {
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
