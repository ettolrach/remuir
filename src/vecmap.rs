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
