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

use std::{fmt::Display, io::{self, Write}, process::ExitCode};

use remuir::{instruction::Instruction, machine::{Identifier, Machine, RuntimeError, TerminationReason}, memory::Memory, parser};
use thiserror::Error;

pub enum ExitStatus {
    Good,
    Error(RemuirError)
}

impl std::process::Termination for ExitStatus {
    fn report(self) -> ExitCode {
        match self {
            Self::Good => ExitCode::from(0),
            Self::Error(e) => {
                println!("{e}");
                ExitCode::from(1)
            }
        }
    }
}

impl From<Result<(), RemuirError>> for ExitStatus {
    fn from(value: Result<(), RemuirError>) -> Self {
        match value {
            Ok(()) => Self::Good,
            Err(e) => Self::Error(e),
        }
    }
}

impl From<io::Result<()>> for ExitStatus {
    fn from(value: io::Result<()>) -> Self {
        match value {
            Ok(()) => Self::Good,
            Err(e) => Self::Error(RemuirError::from(e)),
        }
    }
}

#[derive(Debug, Error)]
pub enum RemuirError {
    #[error("{0}")]
    IOError(#[from] io::Error),
    #[error("Runtime error occurred!\n{0}")]
    RuntimeError(#[from] RuntimeError),
    #[error("Invalid syntax when parsing source code!\n{0}")]
    InvalidSyntax(#[from] parser::ParseSourceError),
    #[error("Can't undo, previous state is unavailable.")]
    CannotUndo,
}

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

    pub fn help_repl() -> io::Result<()> {
        writeln!(io::stdout(), "{}", text_literals::HELP_TEXT_REPL)
    }

    pub fn help_debug() -> io::Result<()> {
        writeln!(io::stdout(), "{}", text_literals::HELP_TEXT_DEBUG)
    }

    pub fn print_prompt() -> io::Result<()> {
        write!(io::stdout(), "\nremuir> ")?;
        io::stdout().flush()?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum Mode {
    Debug { previous_line: Option<usize>, previous_memory: Option<Memory> },
    Repl,
}

impl Mode {
    /// Check if the mode is currently debug.
    pub const fn is_debug(&self) -> bool {
        match self {
            Self::Debug {..} => true,
            Self::Repl => false,
        }
    }

    pub fn set_previous(&mut self, new_line: usize, new_memory: Memory) {
        match self {
            Self::Debug { previous_line, previous_memory } => {
                *previous_line = Some(new_line);
                *previous_memory = Some(new_memory);
            },
            Self::Repl => panic!("Tried to change previous state in REPL mode!"),
        }
    }

    pub fn get_previous(&self) -> Result<(usize, Memory), RemuirError> {
        match self {
            Self::Debug { previous_line, previous_memory } => {
                if previous_line.is_none() || previous_memory.is_none() {
                    return Err(RemuirError::CannotUndo);
                }
                Ok((previous_line.unwrap(), previous_memory.clone().unwrap()))
            },
            Self::Repl => panic!("Tried to access previous state in REPL mode!"),
        }
    }
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Debug {..} => write!(f, "debug"),
            Self::Repl => write!(f, "REPL"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ReplState {
    KeepLooping,
    Stop,
}

#[allow(clippy::too_many_lines)]
pub fn command(input: &str, machine: &mut Machine, mode: &mut Mode) -> Result<ReplState, RemuirError> {
    // Exact matches.
    match input {
        "exit" | "quit" | "q" => {
            printers::goodbye()?;
            return Ok(ReplState::Stop);
        },
        "help" | "h" => {
            match mode {
                Mode::Repl => printers::help_repl()?,
                Mode::Debug { .. } => printers::help_debug()?,
            }
            return Ok(ReplState::KeepLooping);
        },
        "play" | "p" => {
            if !mode.is_debug() {
                writeln!(io::stdout(), "\"play\" is not available in REPL mode.")?;
                return Ok(ReplState::KeepLooping);
            }
            match machine.debug() {
                Ok(TerminationReason::Breakpoint) => {
                    writeln!(io::stdout(), "Reached breakpoint!")?;
                    return Ok(ReplState::KeepLooping);
                },
                Ok(TerminationReason::Empty) => {
                    writeln!(
                        io::stdout(),
                        "Program source code contains no lines of code. Cannot debug an empty program."
                    )?;
                    return Ok(ReplState::KeepLooping);
                },
                Ok(TerminationReason::Halted) => {
                    writeln!(io::stdout(), "Machine successfully halted.")?;
                    return Ok(ReplState::KeepLooping);
                },
                Err(RuntimeError::Halted) => {
                    writeln!(io::stdout(), "Machine is already halted, so cannot step.")?;
                    return Ok(ReplState::KeepLooping);
                },
            }
        },
        "reset" | "r" => {
            machine.reset();
            writeln!(io::stdout(), "Reset machine state!")?;
            return Ok(ReplState::KeepLooping)
        },
        "step" | "s" => {
            if !mode.is_debug() {
                writeln!(io::stdout(), "\"step\" is not available in REPL mode.")?;
                return Ok(ReplState::KeepLooping);
            }
            mode.set_previous(machine.get_current_line_number(), machine.get_state().clone());
            match machine.step() {
                Err(RuntimeError::Halted) => {
                    writeln!(io::stdout(), "Machine is already halted, so cannot step.")?;
                    return Ok(ReplState::KeepLooping)
                },
                Ok(Some(TerminationReason::Halted)) => writeln!(io::stdout(), "Machine successfully halted.")?,
                Ok(None) => (),
                _ => unreachable!(),
            };
            return Ok(ReplState::KeepLooping);
        },
        "undo" | "u" => {
            if !mode.is_debug() {
                writeln!(io::stdout(), "\"undo\" is not available in REPL mode.")?;
                return Ok(ReplState::KeepLooping);
            }
            let (previous_line, previous_memory) = match mode.get_previous() {
                Ok((a, b)) => (a, b),
                Err(e) => {
                    writeln!(io::stdout(), "{e}")?;
                    return Ok(ReplState::KeepLooping);
                },
            };
            machine
                .go_to_identifier(&Identifier::Line(previous_line))
                .expect("Line number must be correct.");
            machine.replace_memory(previous_memory);
            writeln!(io::stdout(), "Undid step.")?;
            return Ok(ReplState::KeepLooping);
        },
        _ => (),
    }

    // Try to parse a memory init line.
    if let Ok(mem) = parser::parse_register_line(input) {
        machine.replace_memory(mem);
        writeln!(io::stdout(), 
            "Registers successfully changed!"
        )?;
        return Ok(ReplState::KeepLooping);
    }
    // Match the start of the input to find the right command.
    let mut input_split = input.split(' ');
    match input_split.next() {
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
                        writeln!(io::stdout(), "Register was already 0. Not jumping due to being in {mode} mode.")?;
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
        Some("breakpoint" | "break" | "b") => {
            if !mode.is_debug() {
                writeln!(io::stdout(), "\"step\" is not available in REPL mode.")?;
                return Ok(ReplState::KeepLooping);
            }
            let Some(ident) = get_ident(input_split)? else { return Ok(ReplState::KeepLooping) };
            match machine.toggle_breakpoint(&ident) {
                Ok(()) => writeln!(io::stdout(), "Successfully added breakpoint.")?,
                Err(e) => {
                    writeln!(io::stdout(), "{e}")?;
                },
            };
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

fn get_ident<'a>(mut iter: impl Iterator<Item = &'a str>) -> Result<Option<Identifier>, RemuirError> {
    let Some(next) = iter.next() else {
        writeln!(
            io::stdout(),
            "Please provide a label or line number to attach a breakpoint to.",
        )?;
        return Ok(None)
    };
    let ident: Identifier;
    // Check if a line number is specified.
    if next.chars().all(|c| c.is_ascii_digit()) {
        let Ok(num) = next.parse::<usize>() else {
            writeln!(
                io::stdout(),
                "Line number too large to attach breakpoint. Must be <={}.",
                usize::MAX,
            )?;
            return Ok(None)
        };
        ident = Identifier::Line(num);
    }
    else if next.to_lowercase().as_str() == "halt" {
        writeln!(io::stdout(), "Cannot use HALT as a breakpoint label.")?;
        return Ok(None);
    }
    // Reconstruct label since it can include spaces.
    else {
        let mut label = String::from(next);
        for s in iter {
            label.push(' ');
            label.push_str(s);
        }
        ident = Identifier::Label(label);
    }
    Ok(Some(ident))
}
