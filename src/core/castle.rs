use super::gamestate;
use super::types;

pub struct Castle {
    pub food_infrastructure: FoodInfrastructure,
}

pub struct FoodInfrastructure {
    pub food_storage: types::Millis,
    pub acres_of_farmland: i32,
}

impl Castle {
    pub fn init(spec: &gamestate::GameSpec) -> Castle {
        return Castle {
            food_infrastructure: FoodInfrastructure {
                food_storage: types::Millis::from_i32(50),
                // 1 acre per character
                acres_of_farmland: spec.initial_characters,
            },
        };
    }
}
