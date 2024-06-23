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

//! Helper functions for interactive modes ("tui").

use std::io::{self, Write};

use remuir::{instruction::Instruction, machine::Machine, parser};

pub mod printers {
    //! Functions which print commonly used and long texts.

    use std::io::{self, Write};
    use crate::text_literals;
    
    /// Print a message when quitting an interactive mode.
    /// 
    /// Currently, it just prints a newline, but in the future this could have a goodbye message.
    pub fn goodbye() -> io::Result<()> {
        writeln!(io::stdout())
    }

    pub fn help() -> io::Result<()> {
        writeln!(io::stdout(), "{}", text_literals::HELP_TEXT)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ReplState {
    KeepLooping,
    Stop,
}

pub fn repl_command(input: &str, machine: &mut Machine) -> io::Result<ReplState> {
    // Exact matches.
    match input {
        "exit" | "quit" | "q" => {
            printers::goodbye()?;
            return Ok(ReplState::Stop);
        },
        "help" | "h" => {
            printers::help()?;
            return Ok(ReplState::KeepLooping);
        },
        "registers" | "r" => {
            writeln!(io::stdout(), "{}", machine.display_nat_registers())?;
            return Ok(ReplState::KeepLooping);
        },
        _ => (),
    }

    // Try to parse a memory init line.
    if let Ok(mem) = parser::parse_register_line(input) {
        machine.replace_memory(mem);
        writeln!(io::stdout(), 
            "Registers successfully changed. Current state:\n{}",
            machine.display_nat_registers()
        )?;
        return Ok(ReplState::KeepLooping);
    }
    // Match the start of the input to find the right command.
    match input.split(' ').next() {
        Some("inc") => {
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
        },
        Some("decjz") => {
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
        },
        Some("dec") => {
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
        },
        _ => {
            writeln!(io::stdout(), "Unknown command \"{input}\". Type \"help\" for a list of commands.")?;
            if input.starts_with("register ") {
                writeln!(io::stdout(), "Note: \"register\" is close to \"registers\".")?;
            }
        }
    }
    Ok(ReplState::KeepLooping)
}
