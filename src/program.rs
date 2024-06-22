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

use crate::{ instruction::Instruction, memory::Memory, vecmap::VecMap };

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
    #[error("Cannot go to line number {line_num}! Last line of program is {last_line}.")]
    LineNumberTooBig { line_num: usize, last_line: usize },
}

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Cannot execute a step, the machine has halted.")]
    Halted,
}

#[derive(Debug, PartialEq)]
pub struct Program {
    lines: Vec<Line>,
    current_line: LineNumber,
    initial_memory: Memory,
    memory: Memory,
    labels: VecMap<String, LineNumber>,
}

impl Program {
    // Constructors.

    /// Construct a new program from a slice of [`Line`]s.
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
            initial_memory: memory.clone(),
            memory,
            labels: labels_map,
        }
    }

    // Editing.

    /// Try to add a new label to a given line number.
    /// 
    /// # Errors
    /// 
    /// * [`ProgramEditError::LabelAlreadyExists`] - returned if a label already exists.
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
    /// * [`ProgramEditError::LineNumberTooBig`] - returned when the line number given is larger
    /// than the last line number.
    pub fn go_to_identifier(&mut self, id: &Identifier) -> Result<(), ProgramEditError> {
        match id {
            Identifier::Halt => {
                self.current_line = self.lines.len() + 1;
                Ok(())
            },
            Identifier::Line(n) => {
                if self.lines.len() < *n {
                    self.current_line = *n;
                    Ok(())
                }
                else {
                    Err(ProgramEditError::LineNumberTooBig {
                        line_num: *n,
                        last_line: self.lines.len() - 1,
                    })
                }
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

    /// Resets the state of the machine by returning the memory to its initial state and setting
    /// the instruction pointer to the first instruction line.
    pub fn reset(&mut self) {
        self.memory = self.initial_memory.clone();
        self.current_line = 0;
    }

    // Execution.

    /// Run the program until the machine halts.
    /// 
    /// This will start running from whatever the current instruction is.
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
    /// * [`RuntimeError::Halted`] - returned when trying to step when the machine has halted.
    pub fn step(&mut self) -> Result<(), RuntimeError> {
        if self.current_line >= self.lines.len() {
            return Err(RuntimeError::Halted)
        }
        // Execute the current instruction.
        self.lines[self.current_line]
            .instruction
            .execute(&mut self.memory)
            // If Ok and an identifier was returned, then jump to said identifier.
            .map(|maybe_ident| match maybe_ident {
                Some(ident) => {
                    self.go_to_identifier(&ident).unwrap();
                },
                None => {
                    self.current_line += 1;
                },
            })
    }

    /// Run the current line of code and return the next line to be run (where the instruction
    /// pointer is pointing after the step).
    /// 
    /// # Errors
    /// 
    /// * [`RuntimeError::Halted`] - returned when trying to step when the machine has halted.
    pub fn step_with_line(&mut self) -> Result<&Line, RuntimeError> {
        self.step()?;
        self.lines.get(self.current_line).map_or_else(
            || Err(RuntimeError::Halted),
            Result::Ok,
        )
    }

    /// Run the current line of code, or in other words, take a "step". Does not check if the
    /// program has reached the end.
    fn step_unchecked(&mut self) {
        self.step().unwrap();
    }

    // Getting state.

    /// Get a string representation of the state of the (natural) registers.
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
