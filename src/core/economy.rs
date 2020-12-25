use super::castle;
use super::character;
use super::population;
use super::team;
use super::types;

use crate::itertools::Itertools;

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


pub enum Op { SUM, MULTIPLY }

pub struct TaggedExp {
    pub tag: String,
    pub e: Exp,
}

pub enum Exp {
    Constant{v: f32},
    BinaryExp{op: Op, v1: Box<TaggedExp>, v2: Box<TaggedExp>},
    ArrayExp{op: Op, vs: Vec<TaggedExp>},
}

impl TaggedExp {
    pub fn eval(&self) -> f32{
        return eval_exp(self);
    }
    pub fn stringify(&self, indent: &str) -> String {
        return stringify_exp(self, indent);
    }
}

pub fn eval_exp(e: &TaggedExp) -> f32 {
    match &e.e {
        Exp::Constant{v} => *v,
        Exp::BinaryExp{op, v1, v2} => match op {
            Op::SUM => eval_exp(&v1) + eval_exp(&v2),
            Op::MULTIPLY => eval_exp(&v1) * eval_exp(&v2),
        },
        Exp::ArrayExp{op, vs} => match op {
            Op::SUM => vs.iter().fold(0.0, |acc, x| acc + eval_exp(x)),
            Op::MULTIPLY => vs.iter().fold(1.0, |acc, x| acc * eval_exp(x)),
        },
    }
}

fn stringify_op(op: &Op) -> String {
    match op {
        Op::SUM => "+".to_string(),
        Op::MULTIPLY => "*".to_string(),
    }
}

// TODO(mrjones): indendentation in sub-ops
fn stringify_exp(e: &TaggedExp, indent: &str) -> String {
    let new_indent = format!("{}  ", indent);
    match &e.e {
        Exp::Constant{v} => format!("{} {}", v, e.tag),
        Exp::BinaryExp{op, v1, v2} => format!(
            "{}  {}\n{}{} {}\n{}==========\n{}= {} {}",
            indent,
            stringify_exp(&v1, &new_indent),
            indent,
            stringify_op(&op),
            stringify_exp(&v2, &new_indent),
            indent,
            indent,
            eval_exp(e),
            e.tag),
        Exp::ArrayExp{op, vs} => {
            let lines = vs.iter().map(|e| stringify_exp(e, &new_indent)).join(&format!("\n{}{} ", indent, stringify_op(&op)));
            return format!("  {}\n{}========\n{}= {} {}", lines, indent, indent, eval_exp(e), e.tag);
        },
    }
}


fn team_linear_traits(weights: &std::collections::HashMap<character::Trait, f32>,
                      team: &team::Team,
                      population: &population::Population) -> TaggedExp {
    let mut character_exps = vec![];
    for c in team.members().iter().map(|cid| population.character_with_id(*cid).unwrap()) {
        let mut character_skills = vec![];
        for (t, weight) in weights {
            character_skills.push(TaggedExp{
                e: Exp::Constant{v: weight * (c.get_trait(*t) as f32 - 50.0) / 10.0},
                tag: character::all_trait_infos().get(t).expect("unknown trait").string3.clone(),
            });
        }
        character_exps.push(TaggedExp{
            e: Exp::ArrayExp{op: Op::SUM, vs: character_skills},
            tag: c.name().to_string(),
        });
    }

    return TaggedExp{
        e: Exp::ArrayExp{op: Op::SUM, vs: character_exps},
        tag: "individual skills".to_string(),
    };
}

fn cotenure_exp(team: &team::Team, rapport_tracker: &population::RapportTracker) -> TaggedExp {
    let log_base = 100.0;
    let multiplier = 1.0 / 3.0;

    if team.members().len() < 2 {
        return TaggedExp{
            e: Exp::Constant{v: 1.0},
            tag: "Single-person team".to_string(),
        };
    }

    let mut total_cotenure: i32 = 0;
    let mut num_pairs: i32 = 0;
    for (c1, c2) in team.member_pairs() {
        total_cotenure += rapport_tracker.turns_on_same_team(&c1, &c2);
        num_pairs += 1;
    }

    if total_cotenure == 0 {
        return TaggedExp{
            e: Exp::Constant{v: 1.0},
            tag: "No past experience together".to_string(),
        };
    }

    let average_cotenure: f32 = total_cotenure as f32 / num_pairs as f32;
    let v = 1.0 + (average_cotenure.log(log_base) * multiplier);

    return TaggedExp{
        e: Exp::Constant{v: v},  // TODO: Could expand this more?
        tag: "cotenure".to_string(),
    };
}

fn production_exp(team: &team::Team, population: &population::Population) -> TaggedExp {
    return TaggedExp{
        e: Exp::BinaryExp{
            op: Op::MULTIPLY,
            v1: Box::new(team_linear_traits(
                &maplit::hashmap!{
                    // 'x' boost per 10 points (1 stdev) of trait:
                    character::Trait::Intelligence => 0.05,
                    character::Trait::WorkEthic => 0.1,
                },
                team,
                population)),
            v2: Box::new(cotenure_exp(team, population.rapport_tracker())),
        },
        tag: "production".to_string(),
    }
}


#[cfg(test)]
mod exp_tests {
    #[test]
    fn simple_exp() {
        use super::TaggedExp;
        use super::Exp;
        use super::Op;

        let e = TaggedExp{
            e: Exp::BinaryExp{
                op: Op::MULTIPLY,
                v1: Box::new(TaggedExp{e: Exp::Constant{v: 4.0}, tag: "base".to_string()}),
                v2: Box::new(TaggedExp{
                    e: Exp::ArrayExp{
                        op: Op::SUM,
                        vs: vec![
                            TaggedExp{e: Exp::Constant{v: 1.1}, tag: "skill1".to_string()},
                            TaggedExp{e: Exp::Constant{v: 1.2}, tag: "skill2".to_string()},
                            TaggedExp{e: Exp::Constant{v: 1.3}, tag: "skill3".to_string()},
                        ],
                    },
                    tag: "boost".to_string()}),
            },
            tag: "production".to_string(),
        };
        assert!(false, "\n{}", super::stringify_exp(&e, ""));

    }
}


fn to_millis(i: Explanation<f32>) -> Explanation<types::Millis> {
    return Explanation{
        v: types::Millis::from_f32(i.v),
        text: i.text,
    };
}

pub struct Explanation<T> {
    pub v: T,
    pub text: String,
//    pub exp: Exp,
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
            text: format!("{} -> {}", mid.text, fin.text),
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
        let t = format!("Sum = {}", v);
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
            text: format!("MapReduceCombiner[map=[\n\t{}\n], red=[{}]]", mid_es.join("\n\t"), fin.text),
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
            text: format!("  {} ({})\n* {} ({})\n= {}", e1.v, e1.text, e2.v, e2.text, (e1.v * e2.v)),
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
//    pub produced_per_turn: Explanation<types::Millis>,
    pub production: TaggedExp,
    pub consumed_per_turn: types::Millis,

    pub num_farmers: f32,
    pub acres_of_farmland: f32,
}

pub fn food(farmers: &team::Team, food_infrastructure: &castle::FoodInfrastructure, population: &population::Population) -> FoodEconomy {
    // TODO(mrjones): Consider sublinear (rather than 0) growth once 'farmers > acres of farmland'
//    let base_production: f32 =
//        std::cmp::min(food_infrastructure.acres_of_farmland, farmers.members().len() as i32) as f32;
    return FoodEconomy{
//        produced_per_turn: to_millis(standard_model(population).explain(&(farmers, farmers))),
        production: production_exp(farmers, population),
        // 1.0 per person.. for now
        consumed_per_turn: types::Millis::from_i32(
            population.characters().len() as i32),

        num_farmers: farmers.members().len() as f32,
        acres_of_farmland: food_infrastructure.acres_of_farmland as f32,
//        base_production: base_production,
    };
}
