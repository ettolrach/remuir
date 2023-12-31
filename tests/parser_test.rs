use rmsim::{*, parser::parse_str};

fn example1_string() -> String {
    String::from("registers 1 2 3
inc r4
some_label: decjz r0 HALT
decjz -1 some_label")
}
fn example1_program() -> Program {
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
    let prog = parse_str(&example1_string());
    let prog_control = example1_program();
    assert_eq!(prog, prog_control)
}
