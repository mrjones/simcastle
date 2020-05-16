use simcastle_core;

fn main() {
    let spec = simcastle_core::gamestate::GameSpec{
        initial_characters: 3,
    };

    let mut game = simcastle_core::Game::new(spec);

    print_workforce(&game);
    print_state(&game);

    loop {
        /*
        use std::io::Write;

        let mut input = String::new();
        print!("> ");
        std::io::stdout().flush().expect("stdout flush");
        std::io::stdin().read_line(&mut input).expect("stdin::read_to_string");
        let input_lower = input.to_ascii_lowercase();
        let input_array = input_lower.trim_end_matches('\n').split(" ").collect::<Vec<&str>>();
         */
        let input_array = get_input_line("> ");
        if input_array.len() == 0 { continue; }

        match input_array[0].as_str() {
            "help" => println!("<help text goes here>"),
            "q" | "quit" => break,
            "vpop" | "population" => print_workforce(&game),
            "va" | "assignments" => print_assignments(&game),
            "assign" => set_assignment(&input_array, &mut game),
            "t" | "turn" => {
                let prompts = game.mut_state().advance_turn();
                handle_prompts(game.mut_state(), prompts);
                print_state(&game);
            }
            _ => println!("Unknown command: {}", input_array.join(" ")),
        }
    }
}

fn get_input_line(prompt: &str) -> Vec<String> {
    use std::io::Write;
    let mut input = String::new();

    print!("{}", prompt);
    std::io::stdout().flush().expect("stdout flush");
    std::io::stdin().read_line(&mut input).expect("stdin::read_to_string");
    let input_lower = input.to_ascii_lowercase();
    return input_lower.trim_end_matches('\n').split(" ").map(|x| x.to_string()).collect::<Vec<String>>();
}

fn handle_prompts(state: &mut simcastle_core::gamestate::GameState, prompts: Vec<simcastle_core::gamestate::Prompt>) {
    for prompt in prompts {
        match prompt {
            simcastle_core::gamestate::Prompt::AsylumSeeker(c) => {
                println!("Seeking asylum: {}", c.full_debug_string());
                let input_array = get_input_line("Accept? (y/n) > ");
                if input_array.len() > 0 && input_array[0].len() > 0 && input_array[0].chars().next().unwrap() == 'y' {
                    state.accept_asylum_seeker(c);
                }
            },
        }
    }
}

fn set_assignment(args: &Vec<String>, game: &mut simcastle_core::Game) {
    assert_eq!(args[0], "assign");
    if args.len() != 3 {
        println!("Invalid assign: assign <char_id> <job>");
        return;
    }

    let char_id = parse_character_id(&args[1]);
    let job = parse_job(&args[2]);

    char_id.map(|char_id| { job.map(|job| {
        println!("Making character {} into a {:?}", char_id, job);
        game.mut_state().mut_workforce().assign(char_id, job);
    })});
}

fn parse_job(job_str: &str) -> Option<simcastle_core::workforce::Job> {
    match job_str {
        "farmer" => return Some(simcastle_core::workforce::Job::FARMER),
        _ => {
            println!("Uknown job: {}", job_str);
            return None;
        }
    }
}

fn parse_character_id(id_str: &str) -> Option<simcastle_core::character::CharacterId> {
    match id_str.parse::<i64>() {
        Ok(v) => return Some(simcastle_core::character::CharacterId(v)),
        Err(_) => {
            println!("Error parsing character id: {}", id_str);
            return None;
        },
    }

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

fn format_delta(x: i32) -> String {
    if x <= 0 {
        return format!("{}", x);
    } else {
        return format!("+{}", x)
    }
}

fn print_state(game: &simcastle_core::Game) {
    println!("Turn: {}, Food: {}/{} ({})", game.state().turn, game.state().food, game.state().castle().food_storage, format_delta(game.state().food_delta()));
}
