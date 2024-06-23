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

use pest::Parser;
use pest_derive::Parser;
use thiserror::Error;

use crate::{
    instruction::Instruction,
    memory::{ Memory, Register, RegisterNumber },
    machine::{ Identifier, Line, Machine },
};

#[derive(Parser)]
#[grammar = "syntax.pest"]
pub struct RemuirParser;

pub fn parse_register_line(s: &str) -> Result<Memory, ParseSourceError> {
    let register_line = RemuirParser::parse(Rule::register_line, s)
        ?
        .next()
        .expect("Can't fail.");

    Ok(
        register_line
            // Turn into an iterator of Pest Pairs.
            .into_inner()
            // Each rule will be the register initial value, so use a map to make them u128s.
            .map(
                |r| r.as_str().parse::<u128>().expect("Assume r < 2^128.")
            )
            .map(Register::from)
            .collect::<Memory>()
    )
}

#[must_use]
fn parse_label(s: &str) -> Identifier {
    match s.to_lowercase().as_str() {
        "halt" => Identifier::Halt,
        _ => Identifier::Label(s.to_string()),
    }
}

#[derive(Error, Debug)]
pub enum ParseSourceError {
    #[error("Syntax error: invalid machine source code. {0}")]
    SyntaxError(#[from] Box<pest::error::Error<Rule>>),
    #[error("Too many arguments specified. Got {received} but {instruction} expects {expected}.")]
    TooManyArgument {
        received: usize,
        instruction: String,
        expected: usize,
    },
    #[error("Too few arguments specified. Got {received} but {instruction} expects {expected}.")]
    TooFewArguments {
        received: usize,
        instruction: String,
        expected: usize,
    },
    #[error("No initial registers provided. Please make the first line \"registers 0\" if this is intentional.")]
    NoInitialRegisters,
}

impl From<pest::error::Error<Rule>> for ParseSourceError {
    fn from(value: pest::error::Error<Rule>) -> Self {
        ParseSourceError::SyntaxError(Box::new(value))
    }
}

/// Parse an increment line.
/// 
/// # Errors
/// 
/// * [`ParseSourceError::SyntaxError`] - when there's a syntax error in the source code.
pub fn parse_inc(s: &str) -> Result<Instruction, ParseSourceError> {
    let inc = RemuirParser::parse(Rule::inc, s)
        ?
        .next()
        .unwrap()
        .into_inner()
        .next()
        .unwrap();

    let reg_num: RegisterNumber = match inc.as_rule() {
        Rule::pos_register_num => RegisterNumber::Natural(
            inc
                .as_str()
                .parse()
                .expect("Guaranteed by Pest.")
        ),
        Rule::neg_register_num => RegisterNumber::Negative(
            inc
                .as_str()
                .parse()
                .expect("Guaranteed by Pest.")
        ),
        _ => unreachable!(),
    };
    Ok(Instruction::INC(reg_num))
}

pub fn parse_decjz(s: &str) -> Result<Instruction, ParseSourceError> {
    use RegisterNumber as Rnum;
    let decjz = RemuirParser::parse(Rule::decjz, s)
        ?
        .next()
        .unwrap();

    let mut final_register_number = Rnum::Natural(0);
    let mut final_label = Identifier::Halt;

    for rule in decjz.into_inner() {
        match rule.as_rule() {
            Rule::pos_register_num => final_register_number = Rnum::Natural(rule.as_str().parse().unwrap()),
            Rule::neg_register_num => final_register_number = Rnum::Negative(rule.as_str().parse().unwrap()),
            Rule::reference_label => final_label = parse_label(rule.as_str()),
            _ => unreachable!(),
        }
    }
    
    Ok(Instruction::DECJZ(final_register_number, final_label))
}

pub fn parse_instruction_line(s: &str, line_num: usize) -> Result<Line, ParseSourceError> {
    let line = RemuirParser::parse(Rule::instruction_line, s)
        ?
        .next()
        .unwrap();

    let mut id: Option<Identifier> = None;
    let mut instruction = Instruction::INC(RegisterNumber::Natural(0));

    for part in line.into_inner() {
        match part.as_rule() {
            Rule::line_label => {
                let s = part.as_str();
                // We need to remove the colon at the end of the label.
                id = Some(Identifier::Label(s[0..(s.len() - 1)].to_string()));
            },
            Rule::instruction => {
                let instruction_part = part.into_inner().next().unwrap();
                match instruction_part.as_rule() {
                    Rule::inc => {
                        instruction = parse_inc(instruction_part.as_str())?;
                    },
                    Rule::decjz => {
                        instruction = parse_decjz(instruction_part.as_str())?;
                    },
                    _ => unreachable!(),
                }
            },
            _ => unreachable!(),
        }
    }

    Ok(Line::new(line_num, id, instruction))
}

/// Parse a register machine source code and return a [`Machine`] struct if the source code is
/// valid.
/// 
/// # Errors
/// 
/// * [`ParseSourceError::SyntaxError`] - when there's a syntax error in the source code.
/// * [`ParseSourceError::NoInitialRegisters`] - when a machine doesn't have an initial registers.
/// line.
pub fn parse_str(input: &str) -> Result<Machine, ParseSourceError> {
    use ParseSourceError as PSErr;
    let file = match RemuirParser::parse(Rule::file, input) {
        Ok(mut pairs) => pairs.next().expect("Can never fail."),
        Err(e) => {
            if !input.trim().starts_with("registers ") {
                return Err(PSErr::NoInitialRegisters);
            }
            return Err(PSErr::from(e));
        },
    };

    let mut lines: Vec<Line> = Vec::new();
    let mut initial_memory: Result<Memory, PSErr> = Err(PSErr::NoInitialRegisters);
    let mut line_number: usize = 0;

    for line in file.into_inner() {
        match line.as_rule() {
            Rule::register_line => {
                initial_memory = Ok(parse_register_line(line.as_str())?);
            },
            Rule::instruction_line => {
                lines.push(parse_instruction_line(line.as_str(), line_number)?);
                line_number += 1;
            },
            Rule::EOI => (),
            _ => unreachable!(),
        }
    }
    Ok(Machine::new_from_lines(&lines[..], initial_memory?))
}

/// Parse a dec instruction. For REPL mode only.
pub fn parse_dec(s: &str) -> Result<Instruction, ParseSourceError> {
    let dec = RemuirParser::parse(Rule::dec, s)
        ?
        .next()
        .unwrap()
        .into_inner()
        .next()
        .unwrap();

    let reg_num: RegisterNumber = match dec.as_rule() {
        Rule::pos_register_num => RegisterNumber::Natural(
            dec
                .as_str()
                .parse()
                .expect("Guaranteed by Pest.")
        ),
        Rule::neg_register_num => RegisterNumber::Negative(
            dec
                .as_str()
                .parse()
                .expect("Guaranteed by Pest.")
        ),
        _ => unreachable!(),
    };
    Ok(Instruction::DECJZ(reg_num, Identifier::Halt))
}
