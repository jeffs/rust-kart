use std::{env, error, fmt, mem, num, process};

#[derive(Debug)]
struct ParseError {
    what: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.what)
    }
}

impl error::Error for ParseError {}

impl From<num::ParseFloatError> for ParseError {
    fn from(err: num::ParseFloatError) -> Self {
        ParseError {
            what: format!("{err}"),
        }
    }
}

#[derive(Debug)]
enum Term {
    Add,
    Mul,
    Num(f64),
}

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
enum Tree {
    Add(Box<Tree>, Box<Tree>),
    Mul(Box<Tree>, Box<Tree>),
    Num(f64),
}

impl Tree {
    fn eval(self) -> f64 {
        match self {
            Tree::Add(left, right) => left.eval() + right.eval(),
            Tree::Mul(left, right) => left.eval() * right.eval(),
            Tree::Num(n) => n,
        }
    }
}

fn parse_op(c: char) -> Result<Term, ParseError> {
    match c {
        '+' => Ok(Term::Add),
        '*' | 'x' => Ok(Term::Mul),
        _ => Err(ParseError {
            what: format!("{c}: bad operator"),
        }),
    }
}

fn parse_terms(s: &str) -> Result<Vec<Term>, ParseError> {
    let mut terms = Vec::new();
    let mut term = String::new();
    for c in s.chars() {
        if c.is_digit(10) || c == '.' {
            term.push(c);
            continue;
        }
        if !term.is_empty() {
            terms.push(Term::Num(mem::take(&mut term).parse()?));
        }
        if c.is_whitespace() {
            continue;
        }
        terms.push(parse_op(c)?);
    }
    if !term.is_empty() {
        terms.push(Term::Num(term.parse()?));
    }
    Ok(terms)
}

fn parse_syntax_imp(mut terms: Vec<Term>, last: Tree) -> Result<Tree, ParseError> {
    if let Some(term) = terms.pop() {
        match term {
            Term::Add => Ok(Tree::Add(Box::new(parse_syntax(terms)?), Box::new(last))),
            Term::Mul => Ok(Tree::Mul(Box::new(parse_syntax(terms)?), Box::new(last))),
            Term::Num(n) => Err(ParseError {
                what: format!("{n} is not an operator"),
            }),
        }
    } else {
        Ok(last)
    }
}

/// Returns an abstract syntax tree.
fn parse_syntax(mut terms: Vec<Term>) -> Result<Tree, ParseError> {
    // Parse back to front; i.e., last term first.
    if let Some(term) = terms.pop() {
        match term {
            Term::Num(n) => parse_syntax_imp(terms, Tree::Num(n)),
            _ => Err(ParseError {
                what: format!("{term} is not an atom"),
            }),
        }
    } else {
        Err(ParseError {
            what: "input is empty".to_string(),
        })
    }
}

fn parse_args() -> Result<Vec<Term>, ParseError> {
    let mut terms = Vec::new();
    for arg in env::args().skip(1) {
        // TODO: Treat separate args as separate terms.
        terms.extend(parse_terms(&arg)?);
    }
    Ok(terms)
}

fn main() {
    let terms = match parse_args() {
        Ok(terms) => terms,
        Err(err) => {
            eprintln!("error: {err}");
            process::exit(2);
        }
    };
    eprintln!("terms: {:?}", terms);
    let tree = match parse_syntax(terms) {
        Ok(tree) => tree,
        Err(err) => {
            eprintln!("Error: {err}");
            process::exit(3);
        }
    };
    eprintln!("tree: {tree:?}");
    println!("{}", tree.eval());
}
