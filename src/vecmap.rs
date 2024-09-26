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

//! A map using a vec without hashing.
//!
//! Performance should be fast for small lists where the hashing function of [`std::collections::HashMap`] would unnecessarily slow down lookup.
//!
//! # Examples
//! ```
//! use remuir::vecmap::VecMap;
//! let mut us_presidents: VecMap<u8, String> = VecMap::from_slice(&vec![
//!     (43, String::from("George W. Bush")),
//!     (44, String::from("Barack Obama")),
//!     (45, String::from("Donald Trump")),
//!     (46, String::from("Joe Biden")),
//! ]);
//! assert_eq!(None, us_presidents.get(&42));
//! us_presidents.update(42, String::from("Bill Clinton"));
//! assert_eq!("Bill Clinton", us_presidents.get(&42).unwrap());
//! ```
#[derive(Default, Debug, PartialEq, Eq)]
pub struct VecMap<K, V> {
    pub vec: Vec<(K, V)>
}
impl<K, V> VecMap<K, V> {
    #[must_use]
    // The use_self has a false positive here. TODO: file bug to the clippy devs.
    #[expect(clippy::use_self)]
    pub fn from_slice(tuples: &[(K, V)]) -> VecMap<K, V>
    where
        K: Clone + PartialEq,
        V: Clone,
    {
        Self { vec: Vec::from(tuples) }
    }

    #[must_use]
    pub fn get(&self, key: &K) -> Option<&V>
    where
        K: PartialEq,
    {
        self.position(key).map(|i| &self.vec[i].1)
    }

    #[must_use]
    fn position(&self, key: &K) -> Option<usize>
    where
        K: PartialEq,
    {
        for i in 0..(self.vec.len()) {
            if &self.vec[i].0 == key {
                return Some(i)
            }
        }
        None
    }
    pub fn update(&mut self, key: K, value: V)
    where
        K: PartialEq
    {
        match self.position(&key) {
            Some(i) => self.vec[i].1 = value,
            None => self.vec.push((key, value)),
        }
    }
    pub fn update_with_fn(&mut self, key: K, identity: &V, func: impl FnOnce(&V) -> V)
    where
        K: PartialEq
    {
        match self.position(&key) {
            Some(i) => self.vec[i].1 = func(&self.vec[i].1),
            None => self.update(key, func(identity)),
        }
    }

    #[must_use]
    pub fn keys(&self) -> Vec<&K> {
        self.vec.iter().map(|tuple| &tuple.0).collect()
    }

    #[must_use]
    pub fn values(&self) -> Vec<&V> {
        self.vec.iter().map(|tuple| &tuple.1).collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::vecmap::VecMap;
    #[test]
    fn simple_update() {
        let mut us_presidents: VecMap<u8, String> = VecMap::from_slice(&[
            (43, String::from("George W. Bush")),
            (44, String::from("Barack Obama")),
            (45, String::from("Donald Trump")),
            (46, String::from("Joe Biden")),
        ]);
        assert_eq!(None, us_presidents.get(&42));
        us_presidents.update(42, String::from("Bill Clinton"));
        assert_eq!("Bill Clinton", us_presidents.get(&42).unwrap());
    }
}
