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

use tui::printers;
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

        // Handle EOF/Ctrl+D.
        if bytes == 0 {
            printers::goodbye()?;
            break;
        }

        match tui::repl_command(input, &mut machine)? {
            tui::ReplState::KeepLooping => continue,
            tui::ReplState::Stop => break,
        }
    }
    Ok(())
}
