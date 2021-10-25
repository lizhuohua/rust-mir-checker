// This would call `unwrap` on `None`
// https://github.com/aesedepece/scriptful/issues/1

use scriptful::op_systems::pokemon::{pokemon_op_sys, Command::*};
use scriptful::prelude::*;

fn main() {
    let mut machine = Machine::new(&pokemon_op_sys);
    machine.operate(&Item::Operator(Evolute));
}
