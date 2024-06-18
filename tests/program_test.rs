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
    memory::{ Memory, Register, RegisterNumber },
    parser::parse_str,
    program::{ Identifier, Instruction, Line, Program },
};

fn get_example_program() -> Program {
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
fn decjz_executing_correctly() {
    let mut program = get_example_program();
    program.execute();
    assert_eq!(&program.display_nat_registers(), "registers 5 0 0")
}

#[test]
fn copy_between_registers() {
    let source_code = String::from("registers 0 3
loop1: decjz r1 loop2
inc r0
inc r2
decjz r3 loop1
loop2: decjz r2 halt
inc r1
decjz r3 loop2");
    let mut prog: Program = parse_str(&source_code).unwrap();
    prog.execute();
    assert_eq!(prog.display_nat_registers(), "registers 3 3 0 0")
}

#[test]
fn copy_using_negative() {
    let source_code = String::from("registers 0 3
loop1: decjz r1 loop2
inc r0
inc r-2
decjz r-1 loop1
loop2: decjz r-2 halt
inc r1
decjz r-1 loop2");
    let mut prog: Program = parse_str(&source_code).unwrap();
    prog.execute();
    assert_eq!(prog.display_nat_registers(), "registers 3 3")
}

#[test]
fn empty_program() {
    let source_code = String::from("registers 1 2 3");
    let mut prog: Program = parse_str(&source_code).unwrap();
    prog.execute();
    assert_eq!(prog.display_nat_registers(), "registers 1 2 3")
}

#[test]
fn simple_increment() {
    let source_code = String::from("registers 0 3
    inc r0");
    let mut prog: Program = parse_str(&source_code).unwrap();
    prog.execute();
    assert_eq!(prog.display_nat_registers(), "registers 1 3")
}

#[test]
fn empty_lines() {
    let source_code = String::from("registers 0 3
loop1: decjz r1 loop2
inc r0

\t\t\t\t

inc r-2
decjz r-1 loop1
loop2: decjz r-2 halt
inc r1


decjz r-1 loop2");
    let mut prog: Program = parse_str(&source_code).unwrap();
    prog.execute();
    assert_eq!(prog.display_nat_registers(), "registers 3 3")
}

#[test]
fn commented_lines() {
    let source_code = String::from("registers 0 3
loop1: decjz r1 loop2
inc r0
# This is a comment.
inc r-2
decjz r-1 loop1
loop2: decjz r-2 halt
inc r1
decjz r-1 loop2");
    let mut prog: Program = parse_str(&source_code).unwrap();
    prog.execute();
    assert_eq!(prog.display_nat_registers(), "registers 3 3")
}
