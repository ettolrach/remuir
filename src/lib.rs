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

use std::convert::Infallible;
use std::num::ParseIntError;
use std::str::FromStr;

pub mod parser;
pub mod vecmap;
use vecmap::VecMap;

// This vector represents a little endian number of base 2^128.
// So, 2^128 + 64 is vec![64, 1]
#[derive(Debug, PartialEq, Clone)]
pub struct Register (Vec<u128>);

impl Register {
    pub fn new(registers: &[u128]) -> Register {
        Register(Vec::from(registers))
    }
    pub fn new_from_u128(value: u128) -> Register {
        Register(vec![value])
    }
    pub fn inc(&mut self) {
        let mut assigned = false;
        // For each u128::MAX digit, set it to 0 and increase the last digit.
        // For example, 39 in base 10, set the units digit to 0 and the tens digit to +1, so 40.
        for num in &mut self.0 {
            match num {
                &mut u128::MAX => {
                    *num = 0;
                },
                ref n => {
                    *num = **n + 1;
                    assigned = true;
                    break;
                },
            }
        }
        // However, if we didn't actually increase any digit, we need to add a new digit set to 1.
        match assigned {
            false => self.0.push(1),
            true => (),
        }
    }
    pub fn dec(&mut self) {
        // A similar principal to inc() is used here.
        let mut decreased = false;
        // For each 0, set it to u128::MAX and decrease the last digit.
        for num in &mut self.0 {
            match num {
                0 => {
                    *num = u128::MAX;
                },
                ref n => {
                    *num = **n - 1;
                    decreased = true;
                    break;
                },
            }
        }
        // If a digit *was* decreased, check whether it's now 0.
        // If so, remove it (no leading zeros!).
        match decreased {
            true => {
                if self.0.last().unwrap() == &0 {
                    self.0.pop();
                }
            },
            false => (),
        }
    }
    fn is_zero(&self) -> bool {
        (self.0.is_empty()) || (self.0.len() == 1 && self.0[0] == 0)
    }
    fn get_u128(&self) -> u128 {
        match self.0.len() {
            0 => 0,
            _ => self.0[0]
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Memory {
    registers: Vec<Register>,
}

impl Memory {
    pub fn new() -> Memory {
        Memory { registers: Vec::new() }
    }
    pub fn new_from_slice(registers: &[Register]) -> Memory {
        Memory { registers: Vec::from(registers) }
    }
    pub fn new_from_iterator(iter: impl Iterator<Item = Register>) -> Memory {
        Memory { registers: Vec::from_iter(iter) }
    }
    pub fn create_new_registers(&mut self, to: usize) {
        for _ in self.registers.len()..to {
            self.registers.push(Register::new_from_u128(0));
        }
    }
    pub fn inc(&mut self, register_number: &usize) {
        if self.registers.len() <= *register_number {
            self.create_new_registers(*register_number);
            self.registers.push(Register::new_from_u128(1));
        }
        else {
            self.registers[*register_number].inc();
        }
    }
    // This function assumes that the register isn't zero!
    pub fn dec(&mut self, register_number: &usize) {
        self.registers[*register_number].dec();

    }
    pub fn is_zero(&mut self, register_number: &usize) -> bool {
        match self.registers.get(*register_number) {
            Some(reg) => {
                match reg.0.len() <= 1 {
                    true => self.registers[*register_number].is_zero(),
                    false => false,
                }
            },
            None => {
                self.create_new_registers(*register_number + 1);
                true
            },
        }
    }
    pub fn get_registers_as_u128(&self) -> Vec<u128> {
        let mut to_return: Vec<u128> = Vec::new();
        for reg in &self.registers[..] {
            to_return.push(reg.get_u128());
        }
        to_return
    }
}
impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}

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

pub enum RegisterParseError {
    NotInt(ParseIntError),
    MissingR,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RegisterNumber {
    Negative(usize),
    Natural(usize),
}

impl FromStr for RegisterNumber {
    type Err = RegisterParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if &s[0..1] != "r" {
            return Err(RegisterParseError::MissingR)
        }
        if &s[1..2] == "-" {
            match (s[2..s.len()]).parse::<usize>() {
                Ok(num) => Ok(Self::Negative(num)),
                Err(e) => Err(RegisterParseError::NotInt(e)),
            }
        }
        else {
            match (s[1..s.len()]).parse::<usize>() {
                Ok(num) => Ok(Self::Natural(num)),
                Err(e) => Err(RegisterParseError::NotInt(e)),
            }
        }
    }
}

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
    natural_memory: Memory,
    negative_memory: Memory,
    labels: VecMap<String, LineNumber>,
}

impl Program {
    pub fn new() -> Program {
        Program {
            lines: Vec::new(),
            current_line: 0,
            natural_memory: Memory::new(),
            negative_memory: Memory::new(),
            labels: VecMap::new(),
        }
    }
    pub fn new_from_lines(lines_slice: &[Line], memory: Memory) -> Program {
        let mut lines_vec: Vec<Line> = Vec::from(lines_slice);
        let mut labels_map = VecMap::new();
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
            natural_memory: memory,
            negative_memory: Memory::new(),
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
                    match register {
                        RegisterNumber::Natural(n) => self.natural_memory.inc(&n),
                        RegisterNumber::Negative(x) => self.negative_memory.inc(&x),
                    };
                }
                Instruction::DECJZ(register, ident_to_jump_to) => {
                    match register {
                        RegisterNumber::Natural(n) => {
                            if self.natural_memory.is_zero(&n) {
                                self.go_to_identifier(&ident_to_jump_to);
                                continue;
                            }
                            else {
                                self.natural_memory.dec(&n);
                            }
                        },
                        RegisterNumber::Negative(x) => {
                            if self.negative_memory.is_zero(&x) {
                                self.go_to_identifier(&ident_to_jump_to);
                                continue;
                            }
                            else {
                                self.negative_memory.dec(&x);
                            }
                        },
                    };
                },
            }
            self.current_line += 1;
        }
    }

    pub fn get_state(&self) -> String {
        let mut to_return = String::new();
        to_return.push_str("registers");

        let register_vec = self.natural_memory.get_registers_as_u128();
        for n in register_vec {
            to_return.push(' ');
            to_return.push_str(&n.to_string());
        }
        to_return
    }
}
impl Default for Program {
    fn default() -> Self {
        Self::new()
    }
}
