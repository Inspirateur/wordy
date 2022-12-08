use itertools::enumerate;
const AGING: f32 = 0.9;

pub struct TopFreqs<const S: usize, T: Eq + Default = usize> {
    pub data: [(T, f32); S],
}

impl<const S: usize, T: Eq + Default> TopFreqs<S, T> {
    pub fn new() -> Self {
        Self {
            data: core::array::from_fn(|_| (T::default(), 0.))
        }
    }

    pub fn get(&self, entry: &T) -> f32 {
        if let Some((_, v)) = self.data.iter().find(|(key, _)| key == entry) {
            *v
        } else {
            0.
        }
    }

    pub fn age(&mut self) {
        self.data.iter_mut().for_each(|(_, v)| *v *= AGING);
    }

    pub fn add(&mut self, entry: T, value: f32) {
        let mut idx = None;
        for (i, (key, v)) in enumerate(&self.data) {
            if key == &entry {
                idx = Some(i);
                break
            }
            if value > *v {
                idx = Some(i);
            }
        }
        if let Some(i) = idx {
            if self.data[i].0 == entry {
                self.data[i].1 += value;
            } else {
                self.age();
                self.data[i] = (entry, value);
            }
        }
    }
}