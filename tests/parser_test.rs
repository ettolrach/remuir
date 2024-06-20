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

use remuir::{
    instruction::Instruction,
    memory::{ Memory, Register, RegisterNumber },
    parser::parse_str,
    program::{ Identifier, Line, Program },
};

fn example1_string() -> String {
    String::from("registers 1 2 3
inc r4
some_label: decjz r0 HALT
decjz r-1 some_label")
}
fn example1_program() -> Program {
    let lines: Vec<Line> = vec![
        Line::new(0, None, Instruction::INC(RegisterNumber::Natural(4))),
        Line::new(1, Some(Identifier::Label(String::from("some_label"))), Instruction::DECJZ(RegisterNumber::Natural(0), Identifier::Halt)),
        Line::new(2, None, Instruction::DECJZ(RegisterNumber::Negative(1), Identifier::Label(String::from("some_label")))),
    ];
    let memory = Memory::new_from_slice(&[
        Register::new_from_u128(1),
        Register::new_from_u128(2),
        Register::new_from_u128(3),
    ][..]);
    Program::new_from_lines(&lines, memory)
}
fn example2_string() -> String {
    String::from("registers 10 5
loop: decjz r1 halt
decjz r0 halt
decjz r2 loop")
}
fn example2_program() -> Program {
    let lines: Vec<Line> = vec![
        Line::new(0, Some(Identifier::Label(String::from("loop"))), Instruction::DECJZ(RegisterNumber::Natural(1), Identifier::Halt)),
        Line::new(1, None, Instruction::DECJZ(RegisterNumber::Natural(0), Identifier::Halt)),
        Line::new(2, None, Instruction::DECJZ(RegisterNumber::Natural(2), Identifier::Label(String::from("loop")))),
    ];
    let memory = Memory::new_from_slice(&[
        Register::new_from_u128(10),
        Register::new_from_u128(5),
    ][..]);
    Program::new_from_lines(&lines, memory)
}

#[test]
fn parse_correctly() {
    let prog = parse_str(&example1_string()).unwrap();
    let prog_control = example1_program();
    assert_eq!(prog, prog_control)
}

#[test]
fn parse_example2() {
    let prog = parse_str(&example2_string()).unwrap();
    let prog_control = example2_program();
    assert_eq!(prog, prog_control)
}

#[test]
fn new_lines() {
    let input = String::from("
registers 3

beginning: decjz r0 even_halt
decjz r0 odd_halt
decjz r-1 beginning

even_halt: decjz r-1 HALT

odd_halt: inc r0
decjz r-1 HALT
");
    let mut prog = parse_str(&input).unwrap();
    prog.execute();
    let output = prog.display_nat_registers();
    let expected_output = String::from("registers 1");
    assert_eq!(expected_output, output)
}
