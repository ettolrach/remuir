use rmsim::*;

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
fn input_parsed_correctly() {
    let input = b"registers 10 5
loop: decjz r1 HALT
decjz r0 HALT
decjz r2 loop";
    let prog: Program = read_input(&input[..]).unwrap();
    let control_program = get_example_program();
    assert_eq!(prog, control_program)
}

#[test]
fn decjz_executing_correctly() {
    let mut program = get_example_program();
    program.execute();
    assert_eq!(&program.get_state(), "registers 5 0 0")
}

#[test]
fn copy_between_registers() {
    let source_code = b"registers 0 3
loop1: decjz r1 loop2
inc r0
inc r2
decjz r3 loop1
loop2: decjz r2 halt
inc r1
decjz r3 loop2";
    let mut prog: Program = read_input(&source_code[..]).unwrap();
    prog.execute();
    assert_eq!(prog.get_state(), "registers 3 3 0 0")
}

#[test]
fn copy_using_negative() {
    let source_code = b"registers 0 3
loop1: decjz r1 loop2
inc r0
inc r-2
decjz r-1 loop1
loop2: decjz r-2 halt
inc r1
decjz r-1 loop2";
    let mut prog: Program = read_input(&source_code[..]).unwrap();
    prog.execute();
    assert_eq!(prog.get_state(), "registers 3 3")
}

#[test]
fn empty_program() {
    let source_code = b"registers 1 2 3";
    let mut prog: Program = read_input(&source_code[..]).unwrap();
    prog.execute();
    assert_eq!(prog.get_state(), "registers 1 2 3")
}

#[test]
fn simple_increment() {
    let source_code = b"registers 0 3
    inc r0";
    let mut prog: Program = read_input(&source_code[..]).unwrap();
    prog.execute();
    assert_eq!(prog.get_state(), "registers 1 3")
}

#[test]
fn empty_lines() {
    let source_code = b"registers 0 3
loop1: decjz r1 loop2
inc r0

\t\t\t\t

inc r-2
decjz r-1 loop1
loop2: decjz r-2 halt
inc r1


decjz r-1 loop2";
    let mut prog: Program = read_input(&source_code[..]).unwrap();
    prog.execute();
    assert_eq!(prog.get_state(), "registers 3 3")
}

#[test]
fn commented_lines() {
    let source_code = b"registers 0 3
loop1: decjz r1 loop2
inc r0
# This is a comment.
inc r-2
decjz r-1 loop1
loop2: decjz r-2 halt
inc r1
decjz r-1 loop2";
    let mut prog: Program = read_input(&source_code[..]).unwrap();
    prog.execute();
    assert_eq!(prog.get_state(), "registers 3 3")
}
