extern crate codingame_snakebyte;

use codingame_snakebyte::game::{FastWorld, WorldState};
use codingame_snakebyte::input_reader::InputReader;
use std::io;

fn main() {
    let stdin = io::stdin();
    let mut input = InputReader::new(stdin.lock());

    let Some(initial) = input.read_initial_state() else {
        return;
    };

    let world = WorldState::from_initial(initial);

    while let Some(turn) = input.read_turn_state(world.width) {
        let fast_world: FastWorld = FastWorld::from_world(&world, &turn);
        println!("{}", fast_world.moves_to_text());
        // world.apply_turn(&turn);
        // eprintln!("{}", world);
    }
}
