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

/// A list map implemented using [`Vec`].
#[derive(Default, Debug, PartialEq, Eq)]
pub struct VecMap<K, V> {
    vec: Vec<(K, V)>
}
impl<K, V> VecMap<K, V> {
    /// Create a [`VecMap`] from a slice of `(key, value)`.
    /// 
    /// <div class="warning">If the slice contains duplicate entries of the key, then this will be
    /// copied without an error!</div>
    ///
    /// # Example
    ///
    /// ```rust
    /// use frust::vecmap::VecMap;
    /// let pairs: Vec<(usize, bool)> = vec![(0, true), (5, false), (6, false)];
    /// let map: VecMap<usize, bool> = VecMap::from_slice(pairs.as_slice());
    /// assert_eq!(map.get(&0), Some(&true));
    /// ```
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

    /// Get the value corresponding to the given `key`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use frust::vecmap::VecMap;
    /// let pairs: Vec<(usize, bool)> = vec![(0, true), (5, false), (6, false), (0, false)];
    /// let map: VecMap<usize, bool> = VecMap::from_slice(pairs.as_slice());
    /// assert_eq!(map.get(&0), Some(&true));
    /// assert_eq!(map.get(&2), None);
    /// ```
    #[must_use]
    pub fn get(&self, key: &K) -> Option<&V>
    where
        K: PartialEq,
    {
        self.position(key).map(|i| &self.vec[i].1)
    }

    /// Retrieve the position of the `key`.
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
    
    /// Set the key to a value. If the key already exists in the map, update the corresponding
    /// value. Otherwise, add it to the map.
    ///
    /// # Example
    ///
    /// ```rust
    /// use frust::vecmap::VecMap;
    /// let pairs: Vec<(usize, bool)> = vec![(0, true), (5, false), (6, false)];
    /// let mut map: VecMap<usize, bool> = VecMap::from_slice(pairs.as_slice());
    /// map.update(5, true);
    /// map.update(2, true);
    /// 
    /// assert_eq!(map.get(&5), Some(&true));
    /// assert_eq!(map.get(&2), Some(&true));
    /// ```
    pub fn update(&mut self, key: K, value: V)
    where
        K: PartialEq
    {
        match self.position(&key) {
            Some(i) => self.vec[i].1 = value,
            None => self.vec.push((key, value)),
        }
    }
    
    /// If the key exists in the map, apply the function to the value. Otherwise, add the key to the
    /// map with the value resulting from applying the function to the value given by `default`.
    /// 
    /// # Example
    /// ```rust
    /// use frust::vecmap::VecMap;
    /// let pairs: Vec<(usize, bool)> = vec![(0, true), (5, false), (6, false)];
    /// let mut map: VecMap<usize, bool> = VecMap::from_slice(pairs.as_slice());
    /// map.update_with_fn(0, &false, |b| b ^ true);
    /// map.update_with_fn(2, &false, |b| b ^ true);
    ///
    /// assert_eq!(map.get(&0), Some(&false));
    /// assert_eq!(map.get(&2), Some(&true));
    /// ```
    pub fn update_with_fn(&mut self, key: K, default: &V, func: impl FnOnce(&V) -> V)
    where
        K: PartialEq
    {
        match self.position(&key) {
            Some(i) => self.vec[i].1 = func(&self.vec[i].1),
            None => self.update(key, func(default)),
        }
    }

    /// Returns a vec of references to the keys.
    #[must_use]
    pub fn keys(&self) -> Vec<&K> {
        self.vec.iter().map(|tuple| &tuple.0).collect()
    }

    /// Returns a vec of references to the values.
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
