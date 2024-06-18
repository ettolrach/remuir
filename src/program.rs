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
use thiserror::Error;

use crate::{ memory::{ Memory, RegisterNumber }, vecmap::VecMap };

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

#[derive(Error, Debug)]
pub enum ProgramEditError {
    #[error("Failed to add label {label:?}, already exists and points to line {line:?}!")]
    LabelAlreadyExists {
        label: String,
        line: usize,
    },
    #[error("Cannot go to label {label:?}! Label not found in the code.")]
    LabelNotFound { label: String },
}

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Cannot step beyond the end of the program.")]
    EndOfProgram,
}

#[derive(Debug, PartialEq)]
pub struct Program {
    lines: Vec<Line>,
    current_line: LineNumber,
    memory: Memory,
    labels: VecMap<String, LineNumber>,
}

impl Program {
    // Constructors.

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

    // Editing.

    /// Try to add a new label to a given line number.
    /// 
    /// # Errors
    /// 
    /// * [`ProgramEditError::LabelAlreadyExists`] - returned if the label already exists.
    pub fn add_new_label(&mut self, label: String, line_number: usize) -> Result<(), ProgramEditError> {
        if let Some(actual_line_number) = self.labels.get(&label) {
            return Err(ProgramEditError::LabelAlreadyExists {
                label,
                line: *actual_line_number,
            })
        }
        self.labels.update(label, line_number);
        Ok(())
    }

    /// Set the instruction pointer to a given identifier.
    /// 
    /// # Errors
    /// 
    /// * [`ProgramEditError::LabelNotFound`] - returned when the specified label doesn't exist in
    /// the code and couldn't be found.
    pub fn go_to_identifier(&mut self, id: &Identifier) -> Result<(), ProgramEditError> {
        match id {
            Identifier::Halt => {
                self.current_line = self.lines.len() + 1;
                Ok(())
            },
            Identifier::Line(n) => {
                self.current_line = *n;
                Ok(())
            },
            Identifier::Label(s) => { 
                self.current_line = match self.labels.get(s) {
                    Some(&n) => n,
                    None => return Err(ProgramEditError::LabelNotFound { label: s.to_owned() }),
                };
                Ok(())
            },
        }
    }

    // Execution.

    pub fn execute(&mut self) {
        if self.lines.is_empty() {
            return;
        }
        while self.current_line < self.lines.len() {
            self.step_unchecked();
        }
    }

    /// Run the current line of code, or in other words, take a "step".
    /// 
    /// # Errors
    /// 
    /// [`RuntimeError::EndOfProgram`] - returned when trying to step beyond the end of the
    /// program.
    pub fn step(&mut self) -> Result<(), RuntimeError> {
        if self.current_line >= self.lines.len() {
            return Err(RuntimeError::EndOfProgram)
        }
        self.step_unchecked();
        Ok(())

    }

    /// Run the current line of code, or in other words, take a "step". Does not check if the
    /// program has reached the end.
    fn step_unchecked(&mut self) {
        let current_instruction = self.lines[self.current_line].instruction.clone();
        match current_instruction {
            Instruction::INC(register) => {
                self.memory.inc(register);
            },
            Instruction::DECJZ(register, ident_to_jump_to) => {
                if self.memory.is_zero(register) {
                    self
                        .go_to_identifier(&ident_to_jump_to)
                        .expect("Ident will always be valid.");
                    return;
                }
                self.memory.dec(register);
            },
        }
        self.current_line += 1;
    }

    // Getting state.

    /// Display the state of the (natural) registers.
    #[must_use]
    pub fn display_nat_registers(&self) -> String {
        format!("{}", self.memory)
    }

    /// Get the state of all registers.
    #[must_use]
    pub fn get_state(&self) -> &Memory {
        &self.memory
    }
}
