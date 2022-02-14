mod ciphertext;

use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead as _, BufReader};
use std::path::Path;
use std::process::exit;

use ciphertext::{Cipher, Ciphertext, WordSet};

fn load_dict(path: impl AsRef<Path>) -> io::Result<WordSet> {
    let mut lines = WordSet::new();
    for line in BufReader::new(File::open(path)?).lines() {
        lines.insert(line?);
    }
    Ok(lines)
}

const DICT: &str = "/usr/share/dict/british-english";

fn decryptions_imp(args: &[Ciphertext], dict: &WordSet, partial: &Cipher) -> Vec<Vec<String>> {
    let mut decryptions = Vec::new();
    if let Some((first, rest)) = args.split_first() {
        let first_ciphers = first.ciphers_derived(dict, partial);
        if rest.is_empty() {
            let words = first_ciphers.iter().map(|cipher| first.decrypt(cipher));
            decryptions.extend(words.map(|word| vec![word]))
        } else {
            for cipher in first_ciphers {
                let first_word = first.decrypt(&cipher);
                for mut rest_words in decryptions_imp(rest, dict, &cipher) {
                    let mut words = vec![first_word.clone()];
                    words.append(&mut rest_words);
                    decryptions.push(words);
                }
            }
        }
    }
    decryptions
}

fn decryptions<'a>(
    args: impl IntoIterator<Item = Ciphertext<'a>>,
    dict: &WordSet,
) -> Vec<Vec<String>> {
    let args: Vec<Ciphertext> = args.into_iter().collect();
    decryptions_imp(&args, dict, &Cipher::new())
}

fn try_main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    let args = args.iter().map(|arg| Ciphertext(arg));
    let dict = load_dict(DICT)?;
    for decryption in decryptions(args, &dict) {
        println!("{}", decryption.join(" "));
    }
    Ok(())
}

fn main() {
    if let Err(what) = try_main() {
        eprintln!("error: {what}");
        exit(1);
    }
}
