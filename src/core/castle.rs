use super::gamestate;
use super::types;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum Infrastructure {
    AcreOfFarmland,
}

impl Infrastructure {
    pub fn build_cost(&self) -> i32 {
        match self {
            &Infrastructure::AcreOfFarmland => 10,
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct BuildQueue {
    pub queue: Vec<Infrastructure>,
    pub progress: i32,
}

pub struct BuildQueueTurnEndStatus {
    pub items_completed: Vec<Infrastructure>,
    pub progress: i32,
}

impl BuildQueue {
    fn new() -> BuildQueue {
        return BuildQueue {
            queue: vec![],
            progress: 0,
        };
    }

    pub fn turn_end(&self, production: i32) -> BuildQueueTurnEndStatus {
        let new_progress = self.progress + production;
        let (costs, items_completed): (Vec<i32>, Vec<Infrastructure>) = self.queue.iter()
            .scan(0, |acc, item| {
                *acc = *acc + item.build_cost();
                return Some((*acc, item));
            })
            .take_while(|(total_cost, _)| *total_cost < new_progress)
            .unzip();

        return BuildQueueTurnEndStatus {
            items_completed: items_completed,
            progress: new_progress - costs.last().unwrap_or(&0),
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Castle {
    pub food_infrastructure: FoodInfrastructure,

    pub build_queue: BuildQueue,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct FoodInfrastructure {
    pub food_storage: types::Millis,
    pub acres_of_farmland: i32,
}

impl Castle {
    pub fn init(spec: &gamestate::GameSpec) -> Castle {
        return Castle {
            food_infrastructure: FoodInfrastructure {
                food_storage: types::Millis::from_i32(50),
                // 1 acre per character
                acres_of_farmland: spec.initial_characters as i32,
            },
            build_queue: BuildQueue::new(),
        };
    }
}
