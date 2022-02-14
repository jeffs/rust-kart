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
        let line = line?;
        if !line.contains(|c| ['a', 'e', 'i', 'o', 'u', 'y'].contains(&c)) {
            continue;
        }
        lines.insert(line);
    }
    Ok(lines)
}

const DICT: &str = "/usr/share/dict/british-english";

fn print_decryptions_imp(
    args: &[Ciphertext],
    dict: &WordSet,
    partial: &Cipher,
    prefix: &mut Vec<String>,
) {
    if let Some((head, tail)) = args.split_first() {
        let head_ciphers = head.ciphers_derived(dict, partial);
        if tail.is_empty() {
            for cipher in head_ciphers {
                let word = head.decrypt(&cipher);
                if prefix.is_empty() {
                    println!("{word}");
                } else {
                    println!("{} {}", prefix.join(" "), word);
                }
            }
        } else {
            for cipher in head_ciphers {
                let word = head.decrypt(&cipher);
                prefix.push(word);
                print_decryptions_imp(tail, dict, &cipher, prefix);
                prefix.pop();
            }
        }
    }
}

fn print_decryptions<'a>(args: impl IntoIterator<Item = Ciphertext<'a>>, dict: &WordSet) {
    let args: Vec<Ciphertext> = args.into_iter().collect();
    print_decryptions_imp(&args, dict, &Cipher::new(), &mut Vec::new())
}

fn try_main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    let args = args.iter().map(|arg| Ciphertext(arg));
    let dict = load_dict(DICT)?;
    print_decryptions(args, &dict);
    Ok(())
}

fn main() {
    if let Err(what) = try_main() {
        eprintln!("error: {what}");
        exit(1);
    }
}
