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

pub const HELP_TEXT: &str = "REPL specific commands:
exit, quit, q       Quit the REPL.
help, h             Display this help text.
registers, r        Display the current state of the (natural) registers.
registers [NUMBERS] Set the registers to the given state. See README.md for more details.

remuir instructions:
inc r[NUMBER]           Increase the given register by 1.
decjz r[NUMBER] [LABEL] Decrease the given register by 1. The label is ignored in REPL mode.
dec r[NUMBER]           Shorter decrement instruction, only available in REPL mode.";

#[must_use]
pub fn welcome_text() -> String {
    format!("remuir {} in REPL mode. Type \"h\" for help", env!("CARGO_PKG_VERSION"))
}
