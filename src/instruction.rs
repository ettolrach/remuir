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

use crate::{
    memory::{ Memory, RegisterNumber },
    machine::Identifier,
};


#[derive(Debug, PartialEq, Clone)]
pub enum Instruction {
    INC(RegisterNumber),
    DECJZ(RegisterNumber, Identifier)
}

impl Instruction {
    pub fn execute(&self, memory: &mut Memory) -> Option<Identifier> {
        match self {
            Instruction::INC(register) => {
                memory.inc(*register);
            },
            Instruction::DECJZ(register, ident_to_jump_to) => {
                if memory.is_zero(*register) {
                    return Some(ident_to_jump_to.clone());
                }
                memory.dec(*register);
            },
        }
        None
    }
}
