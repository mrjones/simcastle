use super::castle;
use super::character;
use super::population;
use super::team;
use super::types;

pub struct FoodEconomy {
    pub produced_per_turn: types::Millis,
    pub consumed_per_turn: types::Millis,

    pub base_production: f32,
    pub skills_boost: f32,
    pub cotenure_boost: f32,
}

pub fn food(farmers: &team::Team, _food_infrastructure: &castle::FoodInfrastructure, population: &population::Population) -> FoodEconomy {
    let base_production: f32 = farmers.members().len() as f32;
    let mut production: f32 = base_production;

    let mut skill_stdevs: f32 = 0.0;
    for id in farmers.members() {
        let c = population.character_with_id(id.clone()).expect("food_production::character_with_id");

        for t in vec![character::Trait::Intelligence, character::Trait::WorkEthic] {
            skill_stdevs += (c.get_trait(t) as f32 - 50.0) / 10.0;
        }
    }
    let skills_boost = skill_stdevs / 10.0;
    production = production + skills_boost;

    let mut cotenure_boost = 1.0;
    if farmers.members().len() > 1 {
        let mut total_cotenure: i32 = 0;
        let mut num_pairs: i32 = 0;
        for (c1, c2) in farmers.member_pairs() {
            total_cotenure += population.rapport_tracker().turns_on_same_team(&c1, &c2);
            num_pairs += 1;
        }

        let average_cotenure: f32 = total_cotenure as f32 / num_pairs as f32;

        if average_cotenure > 0.0 {
            cotenure_boost = 1.0 + (average_cotenure.log(100.0) / 3.0);
        }
    }
    production *= cotenure_boost;

    return FoodEconomy{
        produced_per_turn: types::Millis::from_f32(production),
        // 1.0 per person.. for now
        consumed_per_turn: types::Millis::from_i32(
            population.characters().len() as i32),

        base_production: base_production,
        skills_boost: skills_boost,
        cotenure_boost: cotenure_boost,
    };
}
