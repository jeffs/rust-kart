use itertools::Itertools;
use std::collections::{HashMap, HashSet};

pub type Cipher = HashMap<char, char>;
pub type WordSet = HashSet<String>;

const ALPHABET: [char; 26] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z',
];

pub struct Ciphertext<'a>(pub &'a str);

impl Ciphertext<'_> {
    pub fn _ciphers(&self, dict: &WordSet) -> Vec<Cipher> {
        self.ciphers_derived(dict, &Cipher::new())
    }

    pub fn ciphers_derived(&self, dict: &WordSet, partial: &Cipher) -> Vec<Cipher> {
        let new_chars = self.0.chars().unique().filter(|c| !partial.contains_key(c));
        let keys: Vec<(usize, char)> = new_chars.enumerate().collect();
        (0..ALPHABET.len())
            // TODO: Filter out indexes that are already in partial.values().
            .combinations(keys.len())
            .flat_map(|combo| {
                let k = combo.len();
                combo.into_iter().permutations(k)
            })
            .map(|perm| {
                let mut cipher = partial.clone();
                cipher.extend(keys.iter().map(|&(i, c)| (c, ALPHABET[perm[i]])));
                cipher
            })
            .filter(|cipher| dict.contains(&self.decrypt(cipher)))
            .collect()
    }

    pub fn decrypt(&self, cipher: &Cipher) -> String {
        self.0
            .chars()
            .map(|c| cipher.get(&c).expect("incomplete cipher"))
            .collect()
    }
}
