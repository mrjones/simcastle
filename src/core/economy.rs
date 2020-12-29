use super::castle;
use super::character;
use super::population;
use super::team;
use super::types;

use crate::itertools::Itertools;

// Food production model nodes
// - Leadership
// - Per-character experience/expertise

#[derive(Clone)]
pub enum Op { SUM, MULTIPLY }

// TODO: Unify TaggedExp and Exp
#[derive(Clone)]
pub struct TaggedExp {
    pub tag: String,
    pub e: Exp,
}

#[derive(Clone)]
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
        Exp::Constant{v} => format!("{:+.3} {}", v, e.tag),
        Exp::BinaryExp{op, v1, v2} => format!(
            "{}  {}\n{}{} {}\n{}==========\n{}= {:+.3} {}",
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
            return format!("  {}\n{}========\n{}= {:+.3} {}", lines, indent, indent, eval_exp(e), e.tag);
        },
    }
}

fn team_linear_traits(weights: &std::collections::HashMap<character::Trait, f32>,
                      team: &team::Team,
                      population: &population::Population,
                      infrastructure: &castle::FoodInfrastructure) -> TaggedExp {
    let mut character_exps = vec![];

    let base_production = if team.members().len() as i32 <=  infrastructure.acres_of_farmland {
        TaggedExp{e: Exp::Constant{v: 1.0}, tag: "base".to_string()}
    } else {
        TaggedExp{
            e: Exp::Constant{
                v: infrastructure.acres_of_farmland as f32 / team.members().len() as f32,
            },
            tag: "base (constrained by acres_of_farmland)".to_string(),
        }
    };

    for c in team.members().iter().map(|cid| population.character_with_id(cid.clone()).unwrap()) {
        let mut character_skills = vec![base_production.clone()];
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

fn food_production(team: &team::Team,
                   population: &population::Population,
                   infrastructure: &castle::FoodInfrastructure) -> TaggedExp {
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
                population,
                infrastructure)),
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
        assert_eq!(4.0 * (1.1 + 1.2 + 1.3), e.eval(), "Error evaluating: {}", e.stringify(""));
    }
}

pub struct FoodEconomy {
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
        production: food_production(farmers, population, food_infrastructure),
        // 1.0 per person.. for now
        consumed_per_turn: types::Millis::from_i32(
            population.characters().len() as i32),

        num_farmers: farmers.members().len() as f32,
        acres_of_farmland: food_infrastructure.acres_of_farmland as f32,
    };
}
