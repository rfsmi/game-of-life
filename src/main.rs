use std::str::FromStr;

use crate::hashlife::HashLife;

mod basic_state;
mod hashlife;
mod p3;
mod universe;

fn main() {
    let mut hl: HashLife = HashLife::from_str(
        "
   oo       o
   o o       o
    o      ooo",
    )
    .unwrap();
    hl.step(1000);
}
