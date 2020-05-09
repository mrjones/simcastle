use simcastle_core;

fn main() {
    let mut g = simcastle_core::Game::new();

    for ref c in g.workforce().population() {
        println!(" - {}", c.full_debug_string());
    }

    println!("Initial Food: {}", g.food_production());
    {
        let workforce = g.mut_workforce();
        let worker_id = workforce.population().iter().nth(0).expect("1 worker").id().clone();
        workforce.assign(worker_id, simcastle_core::workforce::Job::FARMER);
    }
    println!("Final Food: {}", g.food_production());
}
