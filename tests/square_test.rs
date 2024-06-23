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

use remuir::{ parser::parse_str, machine::Machine };

#[test]
fn square() {
    let source_code = String::from("registers 3
# r-1 is reserved for GOTO, r-2 will have a copy of r0 in it.
# First, copy r0 to r-2.
1stcopy1: decjz r0 1stcopy2
inc r-2
inc r-9
decjz r-1 1stcopy1
1stcopy2: decjz r-9 3rdcopy1
inc r0
decjz r-1 1stcopy2

# Now, r0 will be copied to r-3.

3rdcopy1: decjz r0 3rdcopy2
inc r-4
inc r-9
decjz r-1 3rdcopy1
3rdcopy2: decjz r-9 drain
inc r0
decjz r-1 3rdcopy2

# Set r0 to 0, then add for the first time.
drain: decjz r0 addition
decjz r-1 drain

multiplication: decjz r-4 HALT
# Restore the counter:
multiplication_copy1: decjz r-2 multiplication_copy2
inc r-3
inc r-9
decjz r-1 multiplication_copy1
multiplication_copy2: decjz r-9 finished_multiplication_copy
inc r-2
decjz r-1 multiplication_copy2
# and go back to adding:
finished_multiplication_copy: decjz r-1 addition

addition: decjz r-3 multiplication
inc r0
decjz r-1 addition
");
    let mut machine: Machine = parse_str(&source_code).unwrap();
    machine.run();
    assert_eq!(machine.display_nat_registers(), "registers 9")
}
