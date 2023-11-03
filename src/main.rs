use std::io;
use rmsim::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut program = read_input(io::stdin().lock())?;
    program.execute();
    let output = program.get_state();
    print!("{}\n", output);
    Ok(())
}
