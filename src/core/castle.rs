use super::types;

pub struct Castle {
    pub food_storage: types::Millis,
}

impl Castle {
    pub fn new() -> Castle {
        return Castle {
            food_storage: types::Millis::from_i32(50),
        };
    }
}
