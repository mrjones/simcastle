use simcastle_core;

fn prompt_restore() -> bool {
    loop {
        let user_input = get_input_line("(n)ew or (r)estore? ");
        if user_input.len() == 0 { continue }

        if user_input[0] == "r" { return true; }
        if user_input[0] == "n" { return false; }
    }
}

fn begin_game() -> simcastle_core::gamestate::GameState {
    let spec = simcastle_core::gamestate::GameSpec{
        initial_potential_characters: 6,
        initial_characters: 3,
    };

    let setup = simcastle_core::initialsetup::InitialSetup::new(spec);

    println!("Potential team:");
    for c in &setup.character_candidates {
        println!("{}", c.full_debug_string());
    }

    let mut team = std::collections::HashSet::new();
    while team.len() < setup.spec().initial_characters {
        let prompt = format!("Pick {} more: ", setup.spec().initial_characters - team.len());
        for char_id_string in &get_input_line(&prompt) {
            let char_id = parse_character_id(char_id_string).expect(&format!("Could not parse id: {}", char_id_string));
            team.insert(char_id);
        }
    }

    let save_file = std::fs::File::create("/tmp/simcastle.save").expect("creating save file");
    return setup.begin(team, save_file).expect("TODO");
}

fn restore_game() -> simcastle_core::gamestate::GameState {
    return simcastle_core::gamestate::GameState::restore(
        "/tmp/simcastle.save").expect("TODO");
}

fn log_level_as_letter(level: log::Level) -> String {
    match level {
        log::Level::Debug => "D".to_string(),
        log::Level::Error => "E".to_string(),
        log::Level::Info => "I".to_string(),
        log::Level::Trace => "T".to_string(),
        log::Level::Warn => "W".to_string(),
    }
}

fn main() {
    fern::Dispatch::new()
        .format(|out, msg, record| {
            out.finish(format_args!("{} [{}:{:<3}] {}",
                                    log_level_as_letter(record.level()),
                                    record.file().unwrap_or("-"),
                                    record.line().unwrap_or(0),
                                    msg));
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .apply()
        .expect("Setting up logging.");

    let mut game: simcastle_core::gamestate::GameState = if prompt_restore() {
        restore_game()
    } else {
        begin_game()
    };
    print_workforce(&game);
    print_state(&game);

    loop {
        let input_array = get_input_line("> ");
        if input_array.len() == 0 { continue; }

        match input_array[0].as_str() {
            "help" => println!("<help text goes here>"),
            "q" | "quit" => break,
            "vpop" | "population" => print_workforce(&game),
            "vt" | "teams" => print_teams(&game),
            "s" | "state" => print_state(&game),
            "ef" | "explain food" => print_food(&game),
            "eb" | "explain builder" => print_builder(&game),
            "assign" => set_assignment(&input_array, &mut game),
            "b" | "build" => build(&input_array, &mut game),
            "bq" | "buildqueue" => print_build_queue(&game),
            "t" | "turn" => {
                let prompts = game.advance_turn().expect("advance_turn");
                handle_prompts(&mut game, prompts);
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
                loop {
                    let input_array = get_input_line("Accept? (y/n) > ");
                    if input_array.len() > 0 && input_array[0].len() == 1 {
                        match input_array[0].chars().next() {
                            Some('y') => {
                                state.execute_command(
                                    &simcastle_core::gamestate::UserCommand::AddCharacter{
                                        character: c.clone(),
                                    }).expect("execute_command");
                                break;
                            },
                            Some('n') => {
                                break;
                            },
                            _ => {}
                        }
                    }
                    println!("'y' or 'n'");
                }
            },
        }
    }
}

fn set_assignment(args: &Vec<String>, game: &mut simcastle_core::gamestate::GameState) {
    assert_eq!(args[0], "assign");
    if args.len() != 3 {
        println!("Invalid assign: assign <char_id> <job>");
        return;
    }

    let char_id = parse_character_id(&args[1]);
    let job = parse_job(&args[2]);

    char_id.map(|char_id| { job.map(|job| {
        println!("Making character {} into a {:?}", char_id, job);
        game.execute_command(&simcastle_core::gamestate::UserCommand::AssignToTeam{cid: char_id, job: job}).expect("execute_command");
    })});
}

fn build(args: &Vec<String>, game: &mut simcastle_core::gamestate::GameState) {
//    assert_eq!(args[0], "assign");
    if args.len() != 2 {
        println!("Invalid assign: build <infra>");
        return;
    }

    let infra = parse_infra(&args[1]).expect("couldn't parse infra");
    game.execute_command(&simcastle_core::gamestate::UserCommand::AddToBuildQueue{infra: infra}).expect("execute build command");
}

fn parse_infra(infra_str: &str) -> Option<simcastle_core::castle::Infrastructure> {
    match infra_str {
        "farm" => return Some(simcastle_core::castle::Infrastructure::AcreOfFarmland),
        _ => {
            println!("Uknown infra: {}", infra_str);
            return None;
        }
    }
}

fn parse_job(job_str: &str) -> Option<simcastle_core::workforce::Job> {
    match job_str {
        "farmer" => return Some(simcastle_core::workforce::Job::FARMER),
        "builder" => return Some(simcastle_core::workforce::Job::BUILDER),
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

fn print_teams(game: &simcastle_core::gamestate::GameState) {
    for (i, (ref job, ref team)) in game.workforce().teams().enumerate() {
        if i > 0 {
            println!();
        }
        println!("== {:?} ==", job);
        for char_id in team.members() {
            let c = game.population().character_with_id(char_id.clone());
            println!("{:?}", c.unwrap().full_debug_string());
        }
    }
//    println!("[[FARMERS]]");
//    for char_id in game.workforce().farmers().members() {
//        let c = game.population().character_with_id(char_id.clone());
//        println!("{:?}", c.unwrap().full_debug_string());
//    }
}

fn print_workforce(game: &simcastle_core::gamestate::GameState) {
    for ref c in game.population().characters() {
        println!(" - {}", c.full_debug_string());
    }
}

fn format_delta(x: simcastle_core::types::Millis) -> String {
    if x <= simcastle_core::types::Millis::zero() {
        return format!("{}", x);
    } else {
        return format!("+{}", x)
    }
}

fn print_state(game: &simcastle_core::gamestate::GameState) {
    println!("Turn: {}, Food: {}/{} ({}), Build: {}/{} ({})",
             game.turn(),
             game.food(), game.castle().food_infrastructure.food_storage, format_delta(game.food_delta()),
             game.castle().build_queue.progress,
             game.castle().build_queue.queue.first().map(|i| i.build_cost()).unwrap_or(
                 simcastle_core::types::Millis::from_i32(-1)),
             format_delta(game.builder_economy().production.eval()));
}


fn print_food(game: &simcastle_core::gamestate::GameState) {
    let econ = game.food_economy();
    println!("  Produced:\n{}\n",
             econ.production.stringify("    "));
    println!("- Consumed: {:.2}", econ.consumed_per_turn);
    println!("=================");
    println!("= Net:      {:.2}", econ.production.eval() - econ.consumed_per_turn);
}

fn print_builder(game: &simcastle_core::gamestate::GameState) {
    let econ = game.builder_economy();
    println!("Produced:\n{}\n",
             econ.production.stringify("    "));
}

fn print_build_queue(game: &simcastle_core::gamestate::GameState) {
    for item in &game.castle().build_queue.queue {
        println!("{:?} {}", item, item.build_cost());
    }
}
