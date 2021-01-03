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
// Ratings in [0,100], 50 is average, 10 / stddev.
pub struct TraitRating {
    pub value: i32,
    pub capacity: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Character {
    id: CharacterId,
    name: String,
    traits: std::collections::HashMap<Trait, TraitRating>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CharacterDelta {
    pub id: CharacterId,
    pub changed_trait_values: std::collections::HashMap<Trait, i32>,
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

    pub fn get_trait_value(&self, t: Trait) -> i32 {
        return self.traits.get(&t).expect("unexpected character trait").value;
    }

    pub fn get_trait_desc(&self, t: Trait) -> &TraitRating {
        return self.traits.get(&t).expect("unexpected character trait");
    }

    pub fn mut_trait(&mut self, t: Trait) -> &mut TraitRating {
        return self.traits.get_mut(&t).expect("unexpected character trait");
    }

    pub fn full_debug_string(&self) -> String {
        let traits_str = Trait::iter().map(|t| {
            let t_desc = self.get_trait_desc(t);
            return format!("{}:{}/{}", t.string3(), t_desc.value, t_desc.capacity);
        }).collect::<Vec<String>>().join(" ");
        return format!("[{:03}|{:10}] {}", self.id.0, self.name, traits_str);
    }

    pub fn compute_end_of_turn_delta(&self) -> Option<CharacterDelta> {
        let mut deltas: std::collections::HashMap<Trait, i32> = maplit::hashmap!{};
        for (t, current) in &self.traits {
            let mut inc = 0;
            loop {
                if current.capacity == current.value { break }
                let prob = (current.capacity - current.value - inc) as f64 / 1000.0;
                // XXX Test for this?
                // TODO tweak weights
                // TODO non-linear probabiliy?
//                println!("prob = {} = 1 / (({} - {} - {}) * 10)", prob, current.capacity, current.value, inc);
                if rand::thread_rng().gen_bool(prob) {
                    inc = inc + 1
                } else {
                    break;
                }
            }
            if inc > 0 {
                println!("Trait change: cid={} trait={} delta={}", self.id, t.string3(), inc);
                deltas.insert(*t, current.value + inc);
            }
        }

        if deltas.len() > 0 {
            return Some(CharacterDelta{id: self.id, changed_trait_values: deltas});
        } else {
            return None;
        }
    }
}

fn random_stat() -> TraitRating {
    let cap_z_score: f32 = rand::thread_rng().sample(rand_distr::StandardNormal);
    let headroom_z_score: f32 = rand::thread_rng().sample(rand_distr::StandardNormal);

    let capacity = 55 + (10.0 * cap_z_score) as i32;
    let headroom = 10 + (5.0 * headroom_z_score) as i32;

    return TraitRating{
        capacity: capacity,
        value: std::cmp::max(0, std::cmp::min(capacity, capacity - headroom)),
    };
}

fn random_traits() -> std::collections::HashMap<Trait, TraitRating> {
    let mut map: std::collections::HashMap<Trait, TraitRating> = std::collections::HashMap::new();
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
