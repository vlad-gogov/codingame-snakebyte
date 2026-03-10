extern crate codingame_snakebyte;

use codingame_snakebyte::game::WorldState;
use codingame_snakebyte::input_reader::InputReader;
use std::io;

fn main() {
    let stdin = io::stdin();
    let mut input = InputReader::new(stdin.lock());

    let Some(initial) = input.read_initial_state() else {
        return;
    };

    let mut world = WorldState::from_initial(initial);

    while let Some(turn) = input.read_turn_state() {
        world.apply_turn(&turn);
        // eprintln!("{}", world);

        // TODO: compute best action from turn state.
        println!("WAIT");
    }
}
