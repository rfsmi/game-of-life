use std::str::FromStr;

use crate::state::State;

mod hashlife;
mod state;

fn main() {
    let state = State::from_str(
        "
o
o
o    oo",
    )
    .unwrap();
    println!("{state}");
}
