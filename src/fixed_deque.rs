use std::{array::from_fn, collections::HashMap, hash::Hash};


pub struct FixedDeque<T, const N: usize = 1000> {
    data: [T; N],
    pos: usize,
    full: bool,
}

impl<T: Default, const N: usize> FixedDeque<T, N> {
    pub fn new() -> Self {
        FixedDeque {
            data: from_fn(|_| T::default()),
            pos: 0,
            full: false
        }
    }
}

impl<T, const N: usize> FixedDeque<T, N> {
    pub fn push(&mut self, elem: T) {
        self.data[self.pos] = elem;
        self.pos += 1;
        if self.pos >= N {
            self.pos = 0;
            self.full = true;
        }
    }
}

impl<T: Hash + Eq + Clone, const N: usize> FixedDeque<T, N> {
    pub fn counts(&self) -> HashMap<T, usize> {
        let mut res = HashMap::new();
        let len = if self.full { self.data.len() } else { self.pos };
        self.data.iter().take(len).cloned().for_each(|elem| {
            let count = res.entry(elem).or_default();
            *count += 1;
        });
        res
    }
}