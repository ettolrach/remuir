pub const HELP_TEXT: &str = "REPL specific commands:
exit, quit, q       Quit the REPL.
help, h             Display this help text.
registers, r        Display the current state of the (natural) registers.
registers [NUMBERS] Set the registers to the given state. See README.md for more details.

remuir instructions:
inc r[NUMBER]           Increase the given register by 1.
decjz r[NUMBER] [LABEL] Decrease the given register by 1. The label is ignored in REPL mode.
dec r[NUMBER]           Shorter decrement instruction, only available in REPL mode.";

#[must_use]
pub fn welcome_text() -> String {
    format!("remuir {} in REPL mode. Type \"h\" for help", env!("CARGO_PKG_VERSION"))
}
