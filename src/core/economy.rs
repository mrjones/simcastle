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


fn to_millis(i: Explanation<f32>) -> Explanation<types::Millis> {
    return Explanation{
        v: types::Millis::from_f32(i.v),
        text: i.text,
    };
}

pub struct Explanation<T> {
    pub v: T,
    pub text: String,
}

trait Model<InT, OutT> {
    fn explain(&self, input: &InT) -> Explanation<OutT>;
}

struct LinearTraitsModel {
    weights: std::collections::HashMap<character::Trait, f32>,
}

// TODO: & -> std::borrow::Borrow?
impl Model<character::Character, f32> for LinearTraitsModel {
    fn explain(&self, character: &character::Character) -> Explanation<f32> {
        let mut production = 1.0;

        let mut texts = vec![];
        for (t, weight) in &self.weights {
            let delta = weight * (character.get_trait(*t) as f32 - 50.0) / 10.0;
            production += delta;
            texts.push(format!("{:?}:{}", t, delta));
        }

        return Explanation{
            v: production,
            text: format!("{} [{}] {}", character.name(), texts.join(" + "), production),
        };
    }
}

struct CharacterExtractorModel<'a> {
    population: &'a population::Population,
}

// XXX: &character again, to avoid copy
impl <'a> Model<team::Team, Vec<character::Character>> for CharacterExtractorModel<'a> {
    fn explain(&self, team: &team::Team) -> Explanation<Vec<character::Character>> {
        let v: Vec<character::Character> = team.members().iter().map(|cid| -> character::Character {
            let c:  &character::Character = self.population.character_with_id(*cid).unwrap();
            return (*c).clone();
        }).collect(); // XXX unwrap

        let count = v.len();

        return Explanation{
            v: v,
            text: format!("{} team member(s)", count),
        };
    }
}

struct Sequencer<'a, InT, MidT, OutT> {
    m1: Box<dyn Model<InT, MidT> + 'a>,
    m2: Box<dyn Model<MidT, OutT> + 'a>,
}

impl <'a, InT, MidT, OutT> Model<InT, OutT> for Sequencer<'a, InT, MidT, OutT> {
    fn explain(&self, input: &InT) -> Explanation<OutT> {
        let mid: Explanation<MidT> = self.m1.explain(input);
        let fin = self.m2.explain(&mid.v);

        return Explanation{
            v: fin.v,
            text: format!("Sequencer[{} -> {}]", mid.text, fin.text),
        };
    }
}

struct MultiplierReducer { }

impl Model<Vec<f32>, f32> for MultiplierReducer {
    fn explain(&self, input: &Vec<f32>) -> Explanation<f32> {
        let v = input.iter().fold(1.0, |acc, i| acc * i);
        let t = format!("MultiplierReducer = {}", v);
        return Explanation{
            v: v,
            text: t,
        };
    }
}

struct SumReducer { }

impl Model<Vec<f32>, f32> for SumReducer {
    fn explain(&self, input: &Vec<f32>) -> Explanation<f32> {
        let v = input.iter().fold(0.0, |acc, i| acc + i);
        let t = format!("SumReducer = {}", v);
        return Explanation{
            v: v,
            text: t,
        };
    }
}

struct MapReduceCombiner<'a, InT, MidT, OutT> {
    mapper: Box<dyn Model<InT, MidT> + 'a>, // XXX remove ref
    reducer: Box<dyn Model<Vec<MidT>, OutT> + 'a>
}

impl <'a, InT, MidT, OutT> Model<Vec<InT>, OutT>  for MapReduceCombiner<'a, InT, MidT, OutT> {
    fn explain(&self, input: &Vec<InT>) -> Explanation<OutT> {
        let mid: Vec<Explanation<MidT>> = input.iter().map(|i: &InT| self.mapper.explain(i)).collect();
        let mid_es: Vec<String> = mid.iter().map(|e| e.text.clone()).collect();
        let mid_vs: Vec<MidT> = mid.into_iter().map(|e| e.v).collect();


        let fin: Explanation<OutT> = self.reducer.explain(&mid_vs);

        return Explanation{
            v: fin.v,
            text: format!("MapReduceCombiner[map=[{}], red=[{}]]", mid_es.join(","), fin.text),
        };
    }
}

struct SimpleMultiplier<'a, T1, T2> {
    m1: Box<dyn Model<T1, f32> + 'a>,
    m2: Box<dyn Model<T2, f32> + 'a>,
}

impl <'a, T1, T2> Model<(&T1, &T2), f32> for SimpleMultiplier<'a, T1, T2> {
    fn explain(&self, input: &(&T1, &T2)) -> Explanation<f32> {
        let (i1, i2) = input;
        let e1 = self.m1.explain(i1);
        let e2 = self.m2.explain(i2);

        return Explanation{
            v: e1.v * e2.v,
            text: format!("{} ({}) * {} ({}) = {}", e1.v, e1.text, e2.v, e2.text, (e1.v * e2.v)),
        };
    }
}


struct ExplainableCotenureModel<'a> {
    rapport_tracker: &'a population::RapportTracker,
    log_base: f32,
    multiplier: f32,
}

impl <'a> Model<team::Team, f32> for ExplainableCotenureModel<'a> {
    fn explain(&self, team: &team::Team) -> Explanation<f32> {
        if team.members().len() < 2 {
            return Explanation{
                v: 1.0,
                text: "Single-person team".to_string(),
            };
        }

        let mut total_cotenure: i32 = 0;
        let mut num_pairs: i32 = 0;
        for (c1, c2) in team.member_pairs() {
            total_cotenure += self.rapport_tracker.turns_on_same_team(&c1, &c2);
            num_pairs += 1;
        }

        if total_cotenure == 0 {
            return Explanation{
                v: 1.0,
                text: "No past experience together".to_string(),
            };
        }

        let average_cotenure: f32 = total_cotenure as f32 / num_pairs as f32;
        let v = 1.0 + (average_cotenure.log(self.log_base) * self.multiplier);

        return Explanation{
            v: v,
            text: format!("Avg. cotenure: {}", average_cotenure),
        };
    }
}

fn standard_model(pop: &population::Population) -> impl Model<(&team::Team, &team::Team), f32> + '_{
    return SimpleMultiplier{
        m1: Box::new(Sequencer{
            m1: Box::new(CharacterExtractorModel{population: &pop}),
            m2: Box::new(MapReduceCombiner::<character::Character, f32, f32>{
                mapper: Box::new(LinearTraitsModel{weights: maplit::hashmap!{
                    // 0.1 boost per 10 points (1 stdev) of strgenth
                    character::Trait::Intelligence => 0.05,
                    character::Trait::WorkEthic => 0.1,
                }}),
                reducer: Box::new(SumReducer{}),
            }),
        }),
        m2: Box::new(ExplainableCotenureModel{
            rapport_tracker: pop.rapport_tracker(),
            log_base: 100.0,
            multiplier: 1.0 / 3.0,
        }),
    };
}

#[cfg(test)]
mod test {
    #[test]
    fn simple_model() {
        use super::Model;

        let mut ch = super::character::Character::new_random(
            super::character::CharacterId(1));
        ch.set_trait(super::character::Trait::WorkEthic, 60);
        ch.set_trait(super::character::Trait::Intelligence, 60);
        let team = super::team::Team::new_with_ids(maplit::hashset!{ch.id()});

        let pop = super::population::Population::new(vec![ch]);

        let m = super::standard_model(&pop);

        let boost_explanation = m.explain(&(&team, &team));

        assert_eq!(1.2, boost_explanation.v, "{}", boost_explanation.text);
    }
}

pub struct FoodEconomy {
    pub produced_per_turn: Explanation<types::Millis>,
    pub consumed_per_turn: types::Millis,

    pub num_farmers: f32,
    pub acres_of_farmland: f32,
}

pub fn food(farmers: &team::Team, food_infrastructure: &castle::FoodInfrastructure, population: &population::Population) -> FoodEconomy {
    // TODO(mrjones): Consider sublinear (rather than 0) growth once 'farmers > acres of farmland'
//    let base_production: f32 =
//        std::cmp::min(food_infrastructure.acres_of_farmland, farmers.members().len() as i32) as f32;
    return FoodEconomy{
        produced_per_turn: to_millis(standard_model(population).explain(&(farmers, farmers))),
        // 1.0 per person.. for now
        consumed_per_turn: types::Millis::from_i32(
            population.characters().len() as i32),

        num_farmers: farmers.members().len() as f32,
        acres_of_farmland: food_infrastructure.acres_of_farmland as f32,
//        base_production: base_production,
    };
}
