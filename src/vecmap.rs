/* This file is part of remuir.

remuir is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

remuir is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with remuir. If not, see <https://www.gnu.org/licenses/>. */

#[derive(Default, Debug, PartialEq)]
pub struct VecMap<K, V> {
    pub vec: Vec<(K, V)>
}
impl<K, V> VecMap<K, V> {
    pub fn new() -> VecMap<K, V>
    where
        K: PartialEq,
    {
        VecMap { vec: Vec::new() }
    }
    pub fn from_slice(tuples: &[(K, V)]) -> VecMap<K, V>
    where
        K: Clone,
        K: PartialEq,
        V: Clone,
    {
        VecMap { vec: Vec::from(tuples) }
    }
    pub fn get(&self, key: &K) -> Option<&V>
    where
        K: PartialEq,
    {
        match self.position(key) {
            Some(i) => Some(&self.vec[i].1),
            None => None,
        }
    }
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
    pub fn update_with_fn(&mut self, key: K, identity: V, func: impl FnOnce(&V) -> V)
    where
        K: PartialEq
    {
        match self.position(&key) {
            Some(i) => self.vec[i].1 = func(&self.vec[i].1),
            None => self.update(key, func(&identity)),
        }
    }
    pub fn keys(&self) -> Vec<&K> {
        self.vec.iter().map(|tuple| &tuple.0).collect()
    }
    pub fn values(&self) -> Vec<&V> {
        self.vec.iter().map(|tuple| &tuple.1).collect()
    }
}
