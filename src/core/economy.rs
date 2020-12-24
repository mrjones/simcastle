use super::castle;
use super::character;
use super::population;
use super::team;
use super::types;

// Food production model nodes
// - Per-attribute weights ... linear(?)
// - Team co-tenure
// - Leadership
// - Per-character experience/expertise

// TODO: What's the type of "Boost"?
// ApplyBoost(Value, Boost)  => Value
//   Multiply([Boost]) => Boost
//     CotenureModel(Team) => Boost
//     Sum([Boost]) => Boost
//       LinearAttributesModel(Team, Weights) => [Boost]
//   BaseProduction(Team, Infrastructure) => Value

struct Explanation<T> {
    v: T,
    text: String,
}

trait Model<InT, OutT> {
    fn apply(&self, input: &InT) -> OutT;
    fn explain(&self, input: &InT) -> Explanation<OutT>;
}

/*
fn noop_explain<InT, OutT>(child: Explanation<T>, apply_fn: impl Fn(InT) -> OutT) -> Explanation<OutT> {
    return Explanation{
        v: apply_fn(input.v),
        text: format!("noop[{}]", input.text),
    };
}
*/

struct LinearTraitsModel {
    weights: std::collections::HashMap<character::Trait, f32>,
}

// TODO: & -> std::borrow::Borrow?
impl Model<character::Character, f32> for LinearTraitsModel {
    fn apply(&self, character: &character::Character) -> f32 {
        let mut boost_accum = 0.0;
        for (t, weight) in &self.weights {
            boost_accum += weight * (character.get_trait(*t) as f32 - 50.0) / 10.0;
        }
        return 1.0 + boost_accum;
    }

    fn explain(&self, input: &character::Character) -> Explanation<f32> {
        return Explanation{
            v: self.apply(input),
            text: format!("LinearTraits[]"),
        };
    }
}

struct CharacterExtractorModel<'a> {
    population: &'a population::Population,
}

// XXX: &character again, to avoid copy
impl <'a> Model<team::Team, Vec<character::Character>> for CharacterExtractorModel<'a> {
    fn apply(&self, team: &team::Team) -> Vec<character::Character> {
        return team.members().iter().map(|cid| -> character::Character {
            let c:  &character::Character = self.population.character_with_id(*cid).unwrap();
            return (*c).clone();
        }).collect(); // XXX unwrap
    }

    fn explain(&self, input: &team::Team) -> Explanation<Vec<character::Character>> {
        return Explanation{
            v: self.apply(input),
            text: format!("CharacterExtractor"),
        };
    }
}

struct SimpleCombiner<'a, InT, MidT, OutT> {
    m1: &'a dyn Model<InT, MidT>,
    m2: &'a dyn Model<MidT, OutT>,
}

impl <'a, InT, MidT, OutT> Model<InT, OutT> for SimpleCombiner<'a, InT, MidT, OutT> {
    fn apply(&self, input: &InT) -> OutT {
        let mid: MidT = self.m1.apply(input);
        return self.m2.apply(&mid);
    }
    fn explain(&self, input: &InT) -> Explanation<OutT> {
        let mid: Explanation<MidT> = self.m1.explain(input);
        let fin = self.m2.explain(&mid.v);

        return Explanation{
            v: fin.v,
            text: format!("SimpleCombiner[{} -> {}]", mid.text, fin.text),
        };
    }
}

struct MultiplierReducer { }

impl Model<Vec<f32>, f32> for MultiplierReducer {
    fn apply(&self, input: &Vec<f32>) -> f32 {
        return input.iter().fold(1.0, |acc, i| acc * i);
    }
    fn explain(&self, input: &Vec<f32>) -> Explanation<f32> {
        return Explanation{
            v: self.apply(input),
            text: format!("MultiplierReducer"),
        };
    }
}

struct MapReduceCombiner<'a, InT, MidT, OutT> {
    mapper: &'a dyn Model<InT, MidT>, // XXX remove ref
    reducer: &'a dyn Model<Vec<MidT>, OutT>
}

impl <'a, InT, MidT, OutT> Model<Vec<InT>, OutT>  for MapReduceCombiner<'a, InT, MidT, OutT> {
    fn apply(&self, input: &Vec<InT>) -> OutT {
        let mid_v: Vec<MidT> = input.iter().map(|i: &InT| self.mapper.apply(i)).collect();
        return self.reducer.apply(&mid_v);
    }
    fn explain(&self, input: &Vec<InT>) -> Explanation<OutT> {
        let mid: Vec<Explanation<MidT>> = input.iter().map(|i: &InT| self.mapper.explain(i)).collect();
        let mid_es: Vec<String> = mid.iter().map(|e| e.text.clone()).collect();
        let mid_vs: Vec<MidT> = mid.into_iter().map(|e| e.v).collect();


        let fin: Explanation<OutT> = self.reducer.explain(&mid_vs);

        return Explanation{
            v: fin.v,
            text: format!("MapReduceCombiner[cardinality={}, map={}, red={}]", input.len(), mid_es.join(","), fin.text),
        };
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn simple_model() {
        use super::Model;

        let mut ch = super::character::Character::new_random(
            super::character::CharacterId(1));
        ch.set_trait(super::character::Trait::Strength, 60);
        let team = super::team::Team::new_with_ids(maplit::hashset!{ch.id()});

        let pop = super::population::Population::new(vec![ch]);

        let m = super::SimpleCombiner{
            m1: &super::CharacterExtractorModel{population: &pop},
            m2: &super::MapReduceCombiner::<super::character::Character, f32, f32>{
                mapper: &super::LinearTraitsModel{weights: maplit::hashmap!{
                    // 0.1 boost per 10 points (1 stdev) of strgenth
                    super::character::Trait::Strength => 0.1,
                }},
                reducer: &super::MultiplierReducer{},
            },
        };


        //        let boost = m.apply(&team);
        let boost_explanation = m.explain(&team);
        assert_eq!(1.2, boost_explanation.v, "{}", boost_explanation.text);
    }
}


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
