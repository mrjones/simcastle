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

    loop {
        use std::io::Write;

        let mut input = String::new();
        print!("> ");
        std::io::stdout().flush().expect("stdout flush");
        std::io::stdin().read_line(&mut input).expect("stdin::read_to_string");
        let input_lower = input.to_ascii_lowercase();
        let input_array = input_lower.trim_end_matches('\n').split(" ").collect::<Vec<&str>>();
        if input_array.len() == 0 { continue; }

        match input_array[0] {
            "help" => println!("<help text goes here>"),
            "q" | "quit" => break,
            "vpop" | "population" => print_workforce(&game),
            "va" | "assignments" => print_assignments(&game),
            "assign" => set_assignment(&input_array, &mut game),
            "t" | "turn" => {
                game.mut_state().advance_turn();
                print_state(&game);
            }
            _ => println!("Unknown command: {}", input),
        }
    }
}

fn set_assignment(args: &Vec<&str>, game: &mut simcastle_core::Game) {
    assert_eq!(args[0], "assign");
    if args.len() != 3 {
        println!("Invalid assign: assign <char_id> <job>");
        return;
    }

    let char_id;
    match args[1].parse::<i64>() {
        Ok(v) => char_id = v,
        Err(err) => {
            println!("Error parsing assign: {:?}", err);
            return;
        },
    }


    let job = match args[2] {
        "farmer" => simcastle_core::workforce::Job::FARMER,
        _ => {
            println!("Uknown job: {}", args[2]);
            return;
        }
    };

    println!("Making character {} into a {:?}", char_id, job);
    game.mut_state().mut_workforce().assign(
        simcastle_core::character::CharacterId(char_id), job);
}

fn print_assignments(game: &simcastle_core::Game) {
    for (char_id, job) in game.state().workforce().assignments() {
        println!("{} is a {:?}", char_id, job);
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
