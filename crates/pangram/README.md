# Pangram

Pangram is a solver for NY Times [Spelling Bee][] puzzles.

## Usage

```sh
pangram LETTERS [WORDS_FILE]
```

Any uppercase `LETTERS` are considered mandatory.  Each solution is printed
on a separate line, and pangrams (words that use all available `LETTERS`)
are prefixed with asterisks.

The default `WORDS_FILE` is /usr/share/dict/words.  Note that this file may
not match the New York Times' word list.  In particular, it may contain
proper nouns, which you can grep out, as in the following example.

## Example

```sh
$ pangram Eachkmn |rg -v '[A-Z]' |tail -3
  nankeen
* checkman
  henchman
```

[Spelling Bee]: https://www.nytimes.com/puzzles/spelling-bee
