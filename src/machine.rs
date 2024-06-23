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

use std::{ convert::Infallible, fmt::Display, str::FromStr };
use thiserror::Error;

use crate::{ instruction::Instruction, memory::{Memory, RegisterNumber}, vecmap::VecMap };

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

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Label(s) => write!(f, "{s}"),
            Self::Line(n) => write!(f, "{n}"),
            Self::Halt => write!(f, "HALT"),
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

impl Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.id {
            Some(Identifier::Label(label)) => write!(f, "{}    {}: {}", self.line_number, label, self.instruction),
            Some(Identifier::Line(n)) => write!(f, "{}    {}", self.line_number, self.instruction),
            Some(Identifier::Halt) => unreachable!(),
            None => write!(f, "{}    {}", self.line_number, self.instruction),
        }
    }
}

#[derive(Error, Debug)]
pub enum MachineEditError {
    #[error("Failed to add label {label:?}, already exists and points to line {line:?}!")]
    LabelAlreadyExists {
        label: String,
        line: usize,
    },
    #[error("Cannot find label {label:?} in the code!")]
    LabelNotFound { label: String },
    #[error("Cannot go to line number {line_num}! Last line of the machine is {last_line}.")]
    LineNumberTooBig { line_num: usize, last_line: usize },
}

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("Cannot execute a step, the machine has already halted.")]
    Halted,
}

#[derive(Debug)]
pub enum TerminationReason {
    /// A breakpoint was reached.
    Breakpoint,
    /// The program has no lines of instructions.
    Empty,
    /// The program halted successfully.
    Halted,
}

#[derive(Debug, PartialEq, Default)]
pub struct Machine {
    lines: Vec<Line>,
    current_line: LineNumber,
    initial_memory: Memory,
    memory: Memory,
    labels: VecMap<String, LineNumber>,
    breakpoints: Vec<usize>,
}

impl Machine {
    // Constructors.

    /// Construct a new machine from a slice of [`Line`]s.
    #[must_use]
    pub fn new_from_lines(lines_slice: &[Line], memory: Memory) -> Machine {
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
        Machine {
            lines: lines_vec,
            current_line: 0,
            initial_memory: memory.clone(),
            memory,
            labels: labels_map,
            breakpoints: Vec::new(),
        }
    }

    // Editing.

    /// Add a breakpoint if one hasn't been added already, or remove it otherwise.
    /// 
    /// # Errors
    /// 
    /// * [`MachineEditError::LabelNotFound`] - returned when the specified label doesn't exist in
    /// the code and couldn't be found.
    /// * [`MachineEditError::LineNumberTooBig`] - returned when the line number given is larger
    /// than the last line number.
    pub fn toggle_breakpoint(&mut self, id: &Identifier) -> Result<(), MachineEditError> {
        match id {
            Identifier::Label(s) => {
                if let Some(n) = self.labels.get(s) {
                    if self.breakpoints.contains(n) {
                        self.breakpoints.remove(*n);
                    }
                    else {
                        self.breakpoints.push(*n);
                    }
                    Ok(())
                }
                else {
                    Err(MachineEditError::LabelNotFound { label: s.to_owned() })
                }
            },
            Identifier::Line(n) => {
                if self.lines.get(*n).is_none() {
                    return Err(MachineEditError::LineNumberTooBig {
                        line_num: *n,
                        last_line: self.lines.len()
                    });
                }
                if self.breakpoints.contains(n) {
                    self.breakpoints.remove(*n);
                }
                else {
                    self.breakpoints.push(*n);
                }
                Ok(())
            },
            Identifier::Halt => unreachable!(),
        }
    }

    /// Try to add a new label to a given line number.
    /// 
    /// # Errors
    /// 
    /// * [`MachineEditError::LabelAlreadyExists`] - returned if a label already exists.
    pub fn add_new_label(&mut self, label: String, line_number: usize) -> Result<(), MachineEditError> {
        if let Some(actual_line_number) = self.labels.get(&label) {
            return Err(MachineEditError::LabelAlreadyExists {
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
    /// * [`MachineEditError::LabelNotFound`] - returned when the specified label doesn't exist in
    /// the code and couldn't be found.
    /// * [`MachineEditError::LineNumberTooBig`] - returned when the line number given is larger
    /// than the last line number.
    pub fn go_to_identifier(&mut self, id: &Identifier) -> Result<(), MachineEditError> {
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
                    Err(MachineEditError::LineNumberTooBig {
                        line_num: *n,
                        last_line: self.lines.len() - 1,
                    })
                }
            },
            Identifier::Label(s) => { 
                self.current_line = match self.labels.get(s) {
                    Some(&n) => n,
                    None => return Err(MachineEditError::LabelNotFound { label: s.to_owned() }),
                };
                Ok(())
            },
        }
    }

    /// Replace the current memory with the given memory.
    pub fn replace_memory(&mut self, new_memory: Memory) {
        self.memory = new_memory;
    }

    /// Resets the state of the machine by returning the memory to its initial state and setting
    /// the instruction pointer to the first instruction line.
    pub fn reset(&mut self) {
        self.memory = self.initial_memory.clone();
        self.current_line = 0;
    }

    // Execution.

    /// Run the machine until a breakpoint is reached or until it halts.
    /// 
    /// # Errors
    /// 
    /// * [`RuntimeError::Halted`] - returned when trying to run when the machine has halted.
    pub fn debug(&mut self) -> Result<TerminationReason, RuntimeError> {
        if self.lines.is_empty() {
            return Ok(TerminationReason::Empty);
        }
        while self.current_line < self.lines.len()
            && !self.breakpoints.contains(&self.current_line)
        {
            self.step_unchecked();
        }
        if self.current_line >= self.lines.len() {
            Ok(TerminationReason::Halted)
        }
        else {
            Ok(TerminationReason::Breakpoint)
        }

    }

    /// Execute the given instruction.
    pub fn execute(&mut self, instruction: Instruction) -> Option<Identifier> {
        instruction.execute(&mut self.memory)
    }

    /// Run the machine until it halts.
    /// 
    /// This will start running from whatever the current instruction is.
    pub fn run(&mut self) {
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
    pub fn step(&mut self) -> Result<Option<TerminationReason>, RuntimeError> {
        if self.current_line >= self.lines.len() {
            return Err(RuntimeError::Halted)
        }
        // Execute the current instruction.
        match self.lines[self.current_line]
            .instruction
            .execute(&mut self.memory)
        {
            Some(ident) => {
                self.go_to_identifier(&ident).unwrap();
            },
            None => {
                self.current_line += 1;
            },
        }
        if self.current_line >= self.lines.len() {
            return Ok(Some(TerminationReason::Halted))
        }
        Ok(None)
    }

    /// Run the current line of code and return the next line to be run (where the instruction
    /// pointer is pointing after the step).
    /// 
    /// # Errors
    /// 
    /// * [`RuntimeError::Halted`] - returned when trying to step when the machine has halted.
    pub fn step_with_line(&mut self) -> Result<&Line, RuntimeError> {
        let _ = self.step()?;
        self.lines.get(self.current_line).map_or_else(
            || Err(RuntimeError::Halted),
            Result::Ok,
        )
    }

    /// Run the current line of code, or in other words, take a "step". Does not check if the
    /// machine has halted.
    fn step_unchecked(&mut self) {
        self.step().unwrap();
    }

    // Getting state.

    /// Get a string representation of the state of the (natural) registers.
    /// 
    /// # Panics
    /// 
    /// * If the value of any register is larger than 2^128 - 1, then this will panic!
    #[must_use]
    pub fn display_nat_registers(&self) -> String {
        format!("{}", self.memory)
    }

    /// Get a string representation of the state of a specific register.
    /// 
    /// # Panics
    /// 
    /// * If the value of any register is larger than 2^128 - 1, then this will panic!
    pub fn display_register(&self, register_number: RegisterNumber) -> String {
        self.memory.get_register(register_number)
    }

    /// Get the state of all registers.
    /// 
    /// # Panics
    /// 
    /// * If the value of any register is larger than 2^128 - 1, then this will panic!
    #[must_use]
    pub fn get_state(&self) -> &Memory {
        &self.memory
    }

    /// Get the current line number which the instruction pointer is pointing to.
    #[must_use]
    pub fn get_current_line_number(&self) -> usize {
        self.current_line
    }

    /// Check if the machine is halted.
    #[must_use]
    pub fn is_halted(&self) -> bool {
        self.current_line >= self.lines.len()
    }

    /// Get the next line to be executed (i.e. what the instruction pointer is pointing to at the
    /// moment).
    #[must_use]
    pub fn peek_next_line(&self) -> &Line {
        &self.lines[self.current_line]
    }
}
