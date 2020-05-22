use super::character;
use super::team;
use super::types;
use super::workforce;

pub fn food_production(farmers: &team::Team, workforce: &workforce::Workforce) -> types::Millis {
    let mut production: f32 = 0.0;
    for id in farmers.members() {
        let c = workforce.character_with_id(id.clone()).expect("food_production::character_with_id");
        production += 1.0;
        if c.get_trait(character::Trait::INTELLIGENCE) > 60 {
            production += 0.1;
        }
    }

    // 10% boost per std-dev of harmony.
    production = production * (1.0 + farmers.harmony() * 0.1);

    return types::Millis::from_f32(production);
}
