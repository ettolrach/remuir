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

use std::str::FromStr;

// This vector represents a little endian number of base 2^128.
// So, 2^128 + 64 is vec![64, 1]
#[derive(Debug, PartialEq, Clone)]
pub struct Register (Vec<u128>);

impl Register {
    #[must_use]
    pub fn new(registers: &[u128]) -> Register {
        Register(Vec::from(registers))
    }
    #[must_use]
    pub fn new_from_u128(value: u128) -> Register {
        Register(vec![value])
    }
    pub fn inc(&mut self) {
        let mut assigned = false;
        // For each u128::MAX digit, set it to 0 and increase the last digit.
        // For example, 39 in base 10, set the units digit to 0 and the tens digit to +1, so 40.
        for num in &mut self.0 {
            match num {
                &mut u128::MAX => {
                    *num = 0;
                },
                ref n => {
                    *num = **n + 1;
                    assigned = true;
                    break;
                },
            }
        }
        // However, if we didn't actually increase any digit, we need to add a new digit set to 1.
        if !assigned {
            self.0.push(1);
        }
    }
    pub fn dec(&mut self) {
        // A similar principal to inc() is used here.
        let mut decreased = false;
        // For each 0, set it to u128::MAX and decrease the last digit.
        for num in &mut self.0 {
            match num {
                0 => {
                    *num = u128::MAX;
                },
                ref n => {
                    *num = **n - 1;
                    decreased = true;
                    break;
                },
            }
        }
        // If a digit *was* decreased, check whether it's now 0.
        // If so, remove it (no leading zeros!).
        #[allow(clippy::missing_panics_doc)]
        if decreased && self.0.last().expect("Register always has at least one digit") == &0 {
            self.0.pop();
        }
    }
    #[must_use]
    fn is_zero(&self) -> bool {
        (self.0.is_empty()) || (self.0.len() == 1 && self.0[0] == 0)
    }
    #[must_use]
    fn get_u128(&self) -> u128 {
        match self.0.len() {
            0 => 0,
            _ => self.0[0]
        }
    }
}

pub enum RegisterParseError {
    NotInt(std::num::ParseIntError),
    MissingR,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RegisterNumber {
    Negative(usize),
    Natural(usize),
}

impl FromStr for RegisterNumber {
    type Err = RegisterParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if &s[0..1] != "r" {
            return Err(RegisterParseError::MissingR)
        }
        if &s[1..2] == "-" {
            match (s[2..s.len()]).parse::<usize>() {
                Ok(num) => Ok(Self::Negative(num)),
                Err(e) => Err(RegisterParseError::NotInt(e)),
            }
        }
        else {
            match (s[1..s.len()]).parse::<usize>() {
                Ok(num) => Ok(Self::Natural(num)),
                Err(e) => Err(RegisterParseError::NotInt(e)),
            }
        }
    }
}

#[derive(Default, Debug, PartialEq)]
pub struct Memory {
    nat_registers: Vec<Register>,
    neg_registers: Vec<Register>,
}

impl Memory {
    #[must_use]
    pub fn new_from_slice(registers: &[Register]) -> Memory {
        Memory { nat_registers: Vec::from(registers), neg_registers: Vec::new() }
    }
    pub fn create_new_registers(&mut self, to: RegisterNumber) {
        match to {
            RegisterNumber::Natural(n) => {
                for _ in self.nat_registers.len()..n {
                    self.nat_registers.push(Register::new_from_u128(0));
                }
            },
            RegisterNumber::Negative(n) => {
                for _ in self.neg_registers.len()..n {
                    self.neg_registers.push(Register::new_from_u128(0));
                }
            },
        }
    }
    pub fn inc(&mut self, register_number: RegisterNumber) {
        match register_number {
            RegisterNumber::Natural(n) => {
                if self.nat_registers.len() <= n {
                    self.create_new_registers(RegisterNumber::Natural(n));
                    self.nat_registers.push(Register::new_from_u128(1));
                }
                else {
                    self.nat_registers[n].inc();
                }
            },
            RegisterNumber::Negative(n) => {
                if self.neg_registers.len() <= n {
                    self.create_new_registers(RegisterNumber::Negative(n));
                    self.neg_registers.push(Register::new_from_u128(1));
                }
                else {
                    self.neg_registers[n].inc();
                }
            },
        }
    }
    // This function assumes that the register isn't zero!
    pub fn dec(&mut self, register_number: RegisterNumber) {
        match register_number {
            RegisterNumber::Natural(n) => self.nat_registers[n].dec(),
            RegisterNumber::Negative(n) => self.neg_registers[n].dec(),
        }

    }
    #[must_use]
    pub fn is_zero(&mut self, register_number: RegisterNumber) -> bool {
        match register_number {
            RegisterNumber::Natural(n) => {
                if let Some(reg) = self.nat_registers.get(n) {
                    if reg.0.len() <= 1 {
                        self.nat_registers[n].is_zero()
                    }
                    else {
                        false
                    }
                }
                else {
                    self.create_new_registers(RegisterNumber::Natural(n + 1));
                    true
                }
            },
            RegisterNumber::Negative(n) => {
                if let Some(reg) = self.neg_registers.get(n) {
                    if reg.0.len() <= 1 {
                        self.neg_registers[n].is_zero()
                    }
                    else {
                        false
                    }
                }
                else {
                    self.create_new_registers(RegisterNumber::Negative(n + 1));
                    true
                }
            },
        }
    }

    #[must_use]
    pub fn get_nat_registers_as_u128(&self) -> Vec<u128> {
        let mut to_return: Vec<u128> = Vec::new();
        for reg in &self.nat_registers[..] {
            to_return.push(reg.get_u128());
        }
        to_return
    }
}

impl FromIterator<Register> for Memory {
    fn from_iter<T: IntoIterator<Item = Register>>(iter: T) -> Self {
        Memory { nat_registers: Vec::from_iter(iter), neg_registers: Vec::new() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inc_from_max_test() {
        let mut reg = Register::new(&[u128::MAX][..]);
        reg.inc();
        assert_eq!(reg, Register::new(&[0, 1]));
    }

    #[test]
    fn inc_units_max_only_test() {
        let mut reg = Register::new(&[u128::MAX, u128::MAX, 4, ]);
        reg.inc();
        assert_eq!(reg, Register::new(&[0, 0, 5]));
    }

    #[test]
    fn dec_from_0_units() {
        let mut reg = Register::new(&[0, 1]);
        reg.dec();
        assert_eq!(reg, Register::new(&[u128::MAX]));
    }

    #[test]
    fn is_zero_test() {
        let reg = Register::new(&[]);
        let mut mem = Memory::new_from_slice(&[reg]);
        assert!(mem.is_zero(RegisterNumber::Natural(0)))
    }
}