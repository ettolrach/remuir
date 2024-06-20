# remuir

A register machine emulator written in Rust.

## What is a register machine?

A register machine is an abstract machine which is used for similar purposes to a Turing Machine, a theoretical model of computing. It's used in computability theory and some other areas of theoretical computer science.

Although the exact specifications vary from author to author, The following is how a register machine is defined for this emulator. A register machine has:

- A finite number of registers (a bit of memory), each holding a natural number.
- A finite sequence of lines of instructions, each one having an index (called a *line number*) and an optional label. Labels can be any unicode text which does not contain the characters `:` or `NEWLINE` (any of `\n`, `\r\n`, and `\r`). There are two instructions, defined as follows:
  - `inc [REGISTER]`: increment the given register. Since registers are made up of natural numbers, this will never overflow. In practise, this is limited by how much memory the OS will give the emulator.
  - `decjz [REGISTER] [LABEL]`: if the given register is 0, then jump to the given label. If it isn't, then decrement the register. There is a special label which can be jumped to called `HALT` (case insensitive!). If this is jumped to, the program immediately stops execution.

## Building

Like most Rust projects, the building process should be as simple as cloning the project and running `cargo build --release`. This project does not currently use any other build tools, nor does it do anything extraordinary.

## Usage

### Grammar of source code

A source code file for remuir should have the extension `.remuir`.

The grammar of a remuir program is given in the file `src/syntax.pest`. An English explanation of the grammar is given below.

The first line of the source code is an initialisation of registers. It must be present (though in the future, this will hopefully change to allow for macros/imports of other remuir programs). It will begin with the word `registers` and is follows by a space-seperated list of register values. For example, `register 3 8 1` will initialise the program with register 0 set to 3, register 1 set to 8, and register 2 set to 1. **Note, initialisations of registers to a value greater than 2^128 - 1 are not currently supported.**

It is recommended to leave a new line between the register line (described above) and the instruction lines (described below).

Instruction lines are now written, as described in the section 'What is a register machine?' above. Specifically, each line may include a label (which is a unicode string which doesn't include the characters `:`, `\n`, `\r\n`, and `\r`). If it does, then after the label, the character `:` must follow. Then, the instruction follows (either `inc [REGISTER]` or `decjz [REGISTER] [LABEL]`). Lines are separated by a newline character (`\n`, `\r\n`, or `\r`, though Unix-style LF `\n` is preferred).

For the sake of making it easier to write programs, negative registers can be used too, for example: `inc r-2`. The primary purpose of this is to have some scratch space.

Comments may be used, they must start with the character `#`. The program will ignore any comments.

Below is an example, further examples can be found in the `examples` directory.

```
registers 1

# This program will copy the number in register 0 to register 1.
beginning: decjz r0 HALT
inc r1
decjz r-1 beginning
```

### Executing a program

The source code of a program should be fed into remuir using STDIN. remuir will then output the state of all positive registers up to the highest register accessed during execution of the program to STDOUT.

So, you could run a program with the command `./remuir < path/to/program.remuir`. The program could output `registers 1 5 5` to STDOUT.

### Tips for writing programs in remuir

A good use of negative registers is leaving the register `r-1` at 0 for the entire duration of the program. Then you can immediately jump to any line via `decjz r-1 my_label`. You can think of this like writing `goto my_label`.

## Licence

remuir: a register machine emulator written in Rust.
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
along with this program.  If not, see <https://www.gnu.org/licenses/>.
