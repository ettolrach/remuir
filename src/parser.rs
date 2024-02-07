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

use pest::{Parser};
use pest_derive::Parser;
use thiserror::Error;
use super::*;

#[derive(Parser)]
#[grammar = "syntax.pest"]
pub struct RemuirParser;

pub fn parse_register_line(s: &str) -> Memory {
    let register_line = RemuirParser::parse(Rule::register_line, s)
        .expect("Shouldn't fail")
        .next()
        .expect("Can't fail.");

    Memory::new_from_iterator(
        register_line
        // Turn into an iterator of Pest Pairs.
        .into_inner()
        // Each rule will be the register initial value, so use a map to make them u128s.
        .map(
            |r| r.as_str().parse::<u128>().expect("Assume r < 2^128.")
        )
        .map(Register::new_from_u128)
    )
}

fn parse_label(s: &str) -> Identifier {
    match s.to_lowercase().as_str() {
        "halt" => Identifier::Halt,
        _ => Identifier::Label(s.to_string()),
    }
}

#[derive(Error, Debug)]
pub enum ParseSourceError {
    #[error("No known instruction called {0}.")]
    UnknownInstruction(String),
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

pub fn parse_inc(s: &str) -> Instruction {
    let inc = RemuirParser::parse(Rule::inc, s)
        .unwrap()
        .next()
        .unwrap()
        .into_inner()
        .next()
        .unwrap();

    use RegisterNumber as Rnum;
    let reg_num: RegisterNumber = match inc.as_rule() {
        Rule::pos_register_num => Rnum::Natural(inc.as_str().parse().expect("Guaranteed by Pest.")),
        Rule::neg_register_num => Rnum::Negative(inc.as_str().parse().expect("Guaranteed by Pest.")),
        _ => unreachable!(),
    };
    Instruction::INC(reg_num)
}

pub fn parse_decjz(s: &str) -> Instruction {
    use RegisterNumber as Rnum;
    let decjz = RemuirParser::parse(Rule::decjz, s)
        .unwrap()
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
    
    Instruction::DECJZ(final_register_number, final_label)
}

pub fn parse_instruction_line(s: &str, line_num: usize) -> Line {
    let line = RemuirParser::parse(Rule::instruction_line, s)
        .unwrap()
        .next()
        .unwrap();

    let mut id: Option<Identifier> = None;
    let mut instruction = Instruction::INC(RegisterNumber::Natural(0));

    for part in line.into_inner() {
        match part.as_rule() {
            Rule::line_label => {
                let s = part.as_str();
                id = Some(Identifier::Label(s[0..(s.len() - 1)].to_string()))
            },
            Rule::instruction => {
                let instruction_part = part.into_inner().next().unwrap();
                match instruction_part.as_rule() {
                    Rule::inc => {
                        instruction = parse_inc(instruction_part.as_str())
                    },
                    Rule::decjz => {
                        instruction = parse_decjz(instruction_part.as_str())
                    },
                    _ => unreachable!(),
                }
            },
            _ => unreachable!(),
        }
    }

    Line::new(line_num, id, instruction)
}

pub fn parse_str(input: &str) -> Result<Program, ParseSourceError> {
    use ParseSourceError as PSErr;
    let file = RemuirParser::parse(Rule::file, input)
        .expect("NEED TO HANDLE IF THE INPUT IS INVALID!!!")
        .next().expect("Can never fail.");

    let mut lines: Vec<Line> = Vec::new();
    let mut initial_memory: Result<Memory, PSErr> = Err(PSErr::NoInitialRegisters);
    let mut line_number: usize = 0;

    for line in file.into_inner() {
        match line.as_rule() {
            Rule::register_line => {
                initial_memory = Ok(parse_register_line(line.as_str()));
            },
            Rule::instruction_line => {
                lines.push(parse_instruction_line(line.as_str(), line_number));
                line_number += 1;
            },
            Rule::EOI => (),
            _ => unreachable!(),
        }
    }
    Ok(Program::new_from_lines(&lines[..], initial_memory?))
}
