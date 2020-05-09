use simcastle_core;

fn main() {
    let spec = simcastle_core::gamestate::GameSpec{
        initial_characters: 3,
    };

    let mut game = simcastle_core::Game::new(spec);

    for ref c in game.state().workforce().population() {
        println!(" - {}", c.full_debug_string());
    }

    println!("Initial Food: {}", game.food_production());
    {
        let workforce = game.mut_state().mut_workforce();
        let worker_id = workforce.population().iter().nth(0).expect("1 worker").id().clone();
        workforce.assign(worker_id, simcastle_core::workforce::Job::FARMER);
    }
    println!("Final Food: {}", game.food_production());
}
