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

use std::{ convert::Infallible, str::FromStr };
use memory::{ RegisterNumber, Memory };

pub mod memory;
pub mod parser;
pub mod vecmap;

use vecmap::VecMap;


#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Identifier {
    Label(String),
    Line(LineNumber),
    Halt,
}

impl FromStr for Identifier {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "HALT" => Ok(Identifier::Halt),
            _ => Ok(Identifier::Label(String::from(s))),
        }
    }
}

// type RegisterNumber = usize;

#[derive(Debug, PartialEq, Clone)]
pub enum Instruction {
    INC(RegisterNumber),
    DECJZ(RegisterNumber, Identifier)
}

type LineNumber = usize;

#[derive(Debug, PartialEq, Clone)]
pub struct Line {
    line_number: LineNumber,
    id: Option<Identifier>,
    instruction: Instruction,
}
impl Line {
    #[must_use]
    pub fn new(line_number: LineNumber, id: Option<Identifier>, instruction: Instruction) -> Line {
        Line { line_number, id, instruction }
    }
    pub fn change_id(&mut self, new_id: Option<Identifier>) {
        self.id = new_id;
    }
}

struct RuntimeError;

#[derive(Debug, PartialEq)]
pub struct Program {
    lines: Vec<Line>,
    current_line: LineNumber,
    memory: Memory,
    labels: VecMap<String, LineNumber>,
}

impl Program {

    #[must_use]
    pub fn new_from_lines(lines_slice: &[Line], memory: Memory) -> Program {
        let mut lines_vec: Vec<Line> = Vec::from(lines_slice);
        let mut labels_map = VecMap::default();
        // Create a map of labels.
        for l in &lines_vec {
            if let Some(Identifier::Label(s)) = &l.id {
                labels_map.update(s.to_string(), l.line_number);
            }
        }
        // Now replace all labels in DECJZ instructions with line numbers to speed up jumps.
        for l in &mut lines_vec {
            if let Instruction::DECJZ(_, Identifier::Label(s)) = &l.instruction {
                if let Some(new_num) = labels_map.get(s) {
                    l.change_id(Some(Identifier::Line(*new_num)));
                }
            }
        }
        Program {
            lines: lines_vec,
            current_line: 0,
            memory,
            labels: labels_map,
        }
    }

    pub fn go_to_identifier(&mut self, id: &Identifier) {
        match id {
            Identifier::Halt => self.current_line = (self.lines.len() + 1) as LineNumber,
            Identifier::Line(n) => self.current_line = *n,
            Identifier::Label(s) => { 
                self.current_line = *self.labels.get(s).expect("Every line should have a label.");
            },
        }
    }

    pub fn execute(&mut self) {
        if self.lines.is_empty() {
            return;
        }
        while self.current_line < self.lines.len() as LineNumber {
            let current_instruction = self.lines[self.current_line as LineNumber].instruction.clone();
            match current_instruction {
                Instruction::INC(register) => {
                    self.memory.inc(register);
                },
                Instruction::DECJZ(register, ident_to_jump_to) => {
                    if self.memory.is_zero(register) {
                        self.go_to_identifier(&ident_to_jump_to);
                        continue;
                    }
                    self.memory.dec(register);
                },
            }
            self.current_line += 1;
        }
    }

    #[must_use]
    pub fn get_state(&self) -> String {
        let mut to_return = String::new();
        to_return.push_str("registers");

        let register_vec = self.memory.get_nat_registers_as_u128();
        for n in register_vec {
            to_return.push(' ');
            to_return.push_str(&n.to_string());
        }
        to_return
    }
}
