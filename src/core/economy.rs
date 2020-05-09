use super::character;
use super::workforce;

pub fn food_production(workforce: &workforce::Workforce) -> i32 {
    let mut production: i32 = 0;
    for (id, job) in workforce.assignments() {
        if *job == workforce::Job::FARMER {
            let c = workforce.character_with_id(id.clone()).expect("food_production::character_with_id");
            production += 10;
            if c.get_trait(character::Trait::INTELLIGENCE) > 55 {
                production += 1;
            }
        }
    }

    return production;
}
