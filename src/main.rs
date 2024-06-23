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

use remuir::{machine::Machine, parser};

mod text_literals;
mod tui;

use tui::{printers, Mode, RemuirError};
#[allow(clippy::wildcard_imports)]
use text_literals::*;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    repl: bool,
    #[arg(short, long)]
    debug: Option<std::path::PathBuf>,
}

fn main() -> tui::ExitStatus {
    let cli = Cli::parse();
    if cli.repl {
        tui::ExitStatus::from(repl())
    }
    else if let Some(path) = cli.debug {
        tui::ExitStatus::from(debug(path))
    }
    else {
        tui::ExitStatus::from(run())
    }
}

fn run() -> io::Result<()> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    let mut program = parser::parse_str(&buffer).unwrap();
    program.run();
    let output = program.display_nat_registers();
    println!("{output}");
    Ok(())
}

fn repl() -> Result<(), RemuirError> {
    writeln!(io::stdout(), "{}", welcome_repl())?;
    let mut machine = Machine::default();
    let mut mode = Mode::Repl;

    loop {
        writeln!(io::stdout(), "\n{}", machine.display_nat_registers())?;
        write!(io::stdout(), "remuir> ")?;
        io::stdout().flush()?;
        let mut line = String::new();
        let bytes = io::stdin().read_line(&mut line)?;
        let input = line.trim();

        // Handle EOF/Ctrl+D.
        if bytes == 0 {
            printers::goodbye()?;
            break;
        }

        // Handle the command and decide whether to keep looping or not.
        match tui::command(input, &mut machine, &mut mode)? {
            tui::ReplState::KeepLooping => continue,
            tui::ReplState::Stop => break,
        }
    }
    Ok(())
}

fn debug(path: std::path::PathBuf) -> Result<(), RemuirError> {
    writeln!(io::stdout(), "{}", welcome_debug())?;
    
    let source_code: String = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            writeln!(io::stdout(), "Error opening and reading file! {e}")?;
            return Err(RemuirError::IOError(e));
        },
    };
    let mut machine = parser::parse_str(&source_code)?;
    let mut mode = Mode::Debug { previous_line: None, previous_memory: None };

    loop {
        writeln!(io::stdout(), "\n{}", machine.display_nat_registers())?;
        if machine.is_halted() {
            writeln!(io::stdout(), "Next line:\nNone (machine halted).")?;
        }
        else {
            writeln!(io::stdout(), "Next line:\n{}", machine.peek_next_line())?;
        }
        printers::print_prompt()?;
        let mut line = String::new();
        let bytes = io::stdin().read_line(&mut line)?;
        let input = line.trim();

        // Handle EOF/Ctrl+D.
        if bytes == 0 {
            printers::goodbye()?;
            break;
        }

        // Handle the command and decide whether to keep looping or not.
        match tui::command(input, &mut machine, &mut mode)? {
            tui::ReplState::KeepLooping => continue,
            tui::ReplState::Stop => break,
        }
    }
    Ok(())
}
