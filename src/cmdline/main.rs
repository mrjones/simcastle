use simcastle_core;

fn main() {
    println!("Hello, world!");
    let g = simcastle_core::Game::new();

    for ref c in g.characters() {
        println!(" - {}", c.full_debug_string());
    }

}
