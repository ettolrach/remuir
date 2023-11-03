use pest::Parser;
use pest_derive::Parser;
use super::*;

#[derive(Parser)]
#[grammar = "syntax.pest"]
pub struct RemuirParser;

pub fn parse_register_line(s: &str) -> Register {
    let register_line = RemuirParser::parse(Rule::register_line, s)
        .expect("Shouldn't fail")
        .next()
        .expect("Can't fail.");

    Register::new_from_iterator(
        register_line
        // Turn into an iterator of Pest Pairs.
        .into_inner()
        // Each rule will be the register initial value, so use a map to make them u128s.
        .map(
            |r| r.as_str().parse::<u128>().expect("Assume r < 2^128.")
        )
    )
}

pub fn parse_str(input: &str) -> Program {
    let file = RemuirParser::parse(Rule::file, input)
        .expect("NEED TO HANDLE IF THE INPUT IS INVALID!!!")
        .next().expect("Can never fail.");

    let mut initial_registers: Vec<u128> = Vec::new();
    let mut lines: Vec<Line> = Vec::new();
    let mut regs: Register;

    for line in file.into_inner() {
        match line.as_rule() {
            Rule::register_line => {
                regs = parse_register_line(line.as_str());
            },
            Rule::instruction_line => {},
            Rule::EOI => (),
            _ => unreachable!(),
        }
    }
    todo!()
}
