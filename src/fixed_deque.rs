use std::{array::from_fn, collections::HashMap, hash::Hash};


pub struct FixedDeque<T, const N: usize = 1000> {
    pub data: [T; N],
    pos: usize
}

impl<T: Default, const N: usize> FixedDeque<T, N> {
    pub fn new() -> Self {
        FixedDeque {
            data: from_fn(|_| T::default()),
            pos: 0
        }
    }
}

impl<T, const N: usize> FixedDeque<T, N> {
    pub fn push(&mut self, elem: T) {
        self.data[self.pos] = elem;
        self.pos += 1;
        if self.pos >= N {
            self.pos = 0;
        }
    }
}

impl<T: Hash + Eq + Clone, const N: usize> FixedDeque<T, N> {
    pub fn counts(&self) -> HashMap<T, usize> {
        let mut res = HashMap::new();
        self.data.iter().cloned().for_each(|elem| {
            let count = res.entry(elem).or_default();
            *count += 1;
        });
        res
    }
}