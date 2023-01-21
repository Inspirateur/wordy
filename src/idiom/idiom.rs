use std::collections::HashMap;
use std::hash::Hash;
use itertools::Itertools;
use bimap::BiMap;
use super::top_freqs::TopFreqs;
use super::text_utils::counts;
const PLACE_VOC_LEN: usize = 500;
const PERSON_VOC_LEN: usize = 200;

pub struct Idioms<P: Hash+Eq, U: Hash+Eq> {
    places: HashMap<P, TopFreqs<PLACE_VOC_LEN>>,
    people: HashMap<U, TopFreqs<PERSON_VOC_LEN>>,
    tokens: BiMap<String, usize>,
}


impl<P: Hash+Eq, U: Hash+Eq> Idioms<P, U> {
    pub fn new() -> Self {
        let mut tokens = BiMap::new();
        // reserve slot 0 for empty string
        tokens.insert(String::new(), 0);
        Self {
            places: HashMap::new(), people: HashMap::new(), tokens
        }
    }

    pub fn update(&mut self, place: P, person: U, tokens: Vec<String>) {
        let place_voc = self.places.entry(place).or_insert(TopFreqs::new());
        let user_voc = self.people.entry(person).or_insert(TopFreqs::new());
        let tokens = counts(tokens);
        for (token, value) in tokens {
            let idx = match self.tokens.get_by_left(&token) {
                Some(v) => *v,
                None => {
                    let v = self.tokens.len();
                    self.tokens.insert(token, v);
                    v
                }
            };
            place_voc.add(idx, value);
            let inctx_value = (-place_voc.get(&idx)).exp()*100.;
            user_voc.add(idx, inctx_value);
        }
    }

    pub fn idiom(&self, person: U) -> Vec<(String, f32)> {
        let res = match self.people.get(&person) {
            Some(voc) => voc.data.clone().into_iter()
                .filter(|(idx, _)| *idx != 0).collect_vec(),
            None => Vec::new()
        };
        res.into_iter()
        .map(|(idx, v)| (self.tokens.get_by_right(&idx).unwrap().clone(), v))
        .collect_vec()
    }
}