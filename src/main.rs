#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use std::io::{self, Read};
use rmsim::parser::parse_str;

fn main() -> std::io::Result<()> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    let mut program = parse_str(&buffer).unwrap();
    program.execute();
    let output = program.get_state();
    println!("{output}");
    Ok(())
}
