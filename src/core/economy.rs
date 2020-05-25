use super::castle;
use super::character;
use super::population;
use super::team;
use super::types;

struct SkillModel {
    linear_weights: std::collections::HashMap<character::Trait, f32>,
}

impl SkillModel {
    fn new(linear_weights: std::collections::HashMap<character::Trait, f32>) -> SkillModel {
        return SkillModel{linear_weights: linear_weights};
    }

    fn character_skill_boost(&self, c: &character::Character) -> f32 {
        let mut skill_stdevs = 0.0;
        for (t, weight) in &self.linear_weights {
            skill_stdevs += weight * (c.get_trait(*t) as f32 - 50.0) / 10.0;
        }
        return skill_stdevs;
    }

    fn team_skill_boost(&self, team: &team::Team, population: &population::Population) -> f32 {
        let mut skills_boost = 0.0;
        for id in team.members() {
            let c = population.character_with_id(id.clone()).expect("food_production::character_with_id");
            skills_boost += self.character_skill_boost(c);
        }
        return skills_boost;
    }
}

struct CotenureModel {
    log_base: f32,
    multiplier: f32,
}

impl CotenureModel {
    pub fn boost(&self, team: &team::Team, rapport_tracker: &population::RapportTracker) -> f32 {
        if team.members().len() < 2 {
            return 1.0;
        }

        let mut total_cotenure: i32 = 0;
        let mut num_pairs: i32 = 0;
        for (c1, c2) in team.member_pairs() {
            total_cotenure += rapport_tracker.turns_on_same_team(&c1, &c2);
            num_pairs += 1;
        }

        if total_cotenure == 0 {
            return 1.0;
        }

        let average_cotenure: f32 = total_cotenure as f32 / num_pairs as f32;

        return 1.0 + (average_cotenure.log(self.log_base) * self.multiplier);
    }
}

pub struct FoodEconomy {
    pub produced_per_turn: types::Millis,
    pub consumed_per_turn: types::Millis,

    pub num_farmers: f32,
    pub acres_of_farmland: f32,
    pub base_production: f32,
    pub skills_boost: f32,
    pub cotenure_boost: f32,
}

pub fn food(farmers: &team::Team, food_infrastructure: &castle::FoodInfrastructure, population: &population::Population) -> FoodEconomy {
    // TODO(mrjones): Consider sublinear (rather than 0) growth once 'farmers > acres of farmland'
    let base_production: f32 =
        std::cmp::min(food_infrastructure.acres_of_farmland, farmers.members().len() as i32) as f32;
    let mut production: f32 = base_production;

    let skill_model = SkillModel::new(maplit::hashmap!{
        character::Trait::Intelligence => 0.05,
        character::Trait::WorkEthic => 0.1,
    });
    let cotenure_model = CotenureModel{
        log_base: 100.0,
        multiplier: 1.0 / 3.0,
    };

    let skills_boost = skill_model.team_skill_boost(farmers, population);
    production = production + skills_boost;

    let cotenure_boost = cotenure_model.boost(farmers, population.rapport_tracker());
    production *= cotenure_boost;

    return FoodEconomy{
        produced_per_turn: types::Millis::from_f32(production),
        // 1.0 per person.. for now
        consumed_per_turn: types::Millis::from_i32(
            population.characters().len() as i32),

        num_farmers: farmers.members().len() as f32,
        acres_of_farmland: food_infrastructure.acres_of_farmland as f32,
        base_production: base_production,
        skills_boost: skills_boost,
        cotenure_boost: cotenure_boost,
    };
}
