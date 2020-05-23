use super::character;
use super::population;
use super::team;
use super::types;

pub fn food_production(farmers: &team::Team, population: &population::Population) -> types::Millis {
    let mut production: f32 = 0.0;
    for id in farmers.members() {
        let c = population.character_with_id(id.clone()).expect("food_production::character_with_id");
        production += 1.0;
        if c.get_trait(character::Trait::Intelligence) > 60 {
            production += 0.1;
        }
    }

    if farmers.members().len() > 1 {
        let mut total_cotenure: i32 = 0;
        let mut num_pairs: i32 = 0;
        for (c1, c2) in farmers.member_pairs() {
            total_cotenure += population.rapport_tracker().turns_on_same_team(&c1, &c2);
            num_pairs += 1;
        }

        let average_tenure: f32 = total_cotenure as f32 / num_pairs as f32;

        if average_tenure > 0.0 {
            let tenure_boost = 1.0 + (average_tenure.log(100.0) / 3.0);
            println!("Base prod: {}. Avg tenure: {}, Tenure boost: {}", production, average_tenure, tenure_boost);

            production *= tenure_boost;
            println!("Boosted prod: {}", production);
        }
    }

    return types::Millis::from_f32(production);
}
