/* remuir: a register machine emulator written in Rust.
Copyright (C) 2024  Charlotte Ausel

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>. */

#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use clap::Parser;

use std::io::{self, Read, Write,};

use remuir::{instruction::Instruction, machine::Machine, parser};

mod text_literals;
#[allow(clippy::wildcard_imports)]
use text_literals::*;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    repl: bool,
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();
    if cli.repl {
        repl()
    }
    else {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        let mut program = parser::parse_str(&buffer).unwrap();
        program.run();
        let output = program.display_nat_registers();
        println!("{output}");
        Ok(())
    }
}

fn repl() -> std::io::Result<()> {
    writeln!(io::stdout(), "{}", welcome_text())?;
    let mut machine = Machine::default();
    loop {
        write!(io::stdout(), "remuir> ")?;
        io::stdout().flush()?;
        let mut line = String::new();
        let bytes = io::stdin().read_line(&mut line)?;
        let input = line.trim();
        if ["exit", "quit", "q"].contains(&input) || bytes == 0 {
            writeln!(io::stdout())?;
            break;
        }
        else if ["help", "h"].contains(&input) {
            writeln!(io::stdout(), "{HELP_TEXT}")?;
        }
        else if ["registers", "r"].contains(&input) {
            writeln!(io::stdout(), "{}", machine.display_nat_registers())?;
        }
        else if let Ok(mem) = parser::parse_register_line(input) {
            machine.replace_memory(mem);
            writeln!(io::stdout(), 
                "Registers successfully changed. Current state:\n{}",
                machine.display_nat_registers()
            )?;
        }
        else if input.starts_with("inc") {
            match parser::parse_inc(input) {
                Ok(Instruction::INC(reg_num)) => {
                    let _ = machine.execute(Instruction::INC(reg_num));
                    writeln!(io::stdout(), "Register {reg_num} is now {}.", machine.display_register(reg_num))?;
                },
                Err(parser::ParseSourceError::SyntaxError(b)) => {
                    writeln!(io::stdout(), "Syntax error:\n{b}")?;
                    writeln!(io::stdout(), "Correct usage: inc r[NUMBER]")?;
                },
                _ => unreachable!(),
            }
        }
        else if input.starts_with("decjz") {
            match parser::parse_decjz(input) {
                Ok(Instruction::DECJZ(reg_num, label)) => {
                    if machine.execute(Instruction::DECJZ(reg_num, label)).is_some() {
                        writeln!(io::stdout(), "Register was already 0. Not jumping due to being in REPL mode.")?;
                    } else {
                        writeln!(io::stdout(), "Register {reg_num} is now {}.", machine.display_register(reg_num))?;
                    }
                },
                Err(parser::ParseSourceError::SyntaxError(b)) => {
                    writeln!(io::stdout(), "Syntax error:\n{b}")?;
                    writeln!(io::stdout(), "Correct usage: decjz r[NUMBER] [LABEL]")?;
                },
                _ => unreachable!(),
            }
        }
        else if input.starts_with("dec") {
            match parser::parse_dec(input) {
                Ok(Instruction::DECJZ(reg_num, label)) => {
                    let _ = machine.execute(Instruction::DECJZ(reg_num, label));
                    writeln!(io::stdout(), "Register {reg_num} is now {}.", machine.display_register(reg_num))?;
                },
                Err(parser::ParseSourceError::SyntaxError(b)) => {
                    writeln!(io::stdout(), "Syntax error:\n{b}")?;
                    writeln!(io::stdout(), "Correct usage: dec r[NUMBER]")?;
                },
                _ => unreachable!(),
            }
        }
        else {
            writeln!(io::stdout(), "Unknown command \"{input}\". Type \"help\" for a list of commands.")?;
            if input.starts_with("register ") {
                writeln!(io::stdout(), "Note: \"register\" is close to \"registers\".")?;
            }
            continue;
        }
    }
    Ok(())
}
