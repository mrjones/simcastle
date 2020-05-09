use simcastle_core;

fn main() {
    let spec = simcastle_core::gamestate::GameSpec{
        initial_characters: 3,
    };

    let mut game = simcastle_core::Game::new(spec);

    print_workforce(&game);

    {
        let workforce = game.mut_state().mut_workforce();
        let worker_id = workforce.population().iter().nth(0).expect("1 worker").id().clone();
        workforce.assign(worker_id, simcastle_core::workforce::Job::FARMER);
    }
    print_state(&game);

    for _ in 0..10 {
        game.mut_state().advance_turn();
        print_state(&game);
    }
}


fn print_workforce(game: &simcastle_core::Game) {
    for ref c in game.state().workforce().population() {
        println!(" - {}", c.full_debug_string());
    }
}

fn print_state(game: &simcastle_core::Game) {
    println!("Turn: {}, Food: {}", game.state().turn, game.state().food);
}
