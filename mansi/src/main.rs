use std::{
    io::{self, Read, Write},
    process::exit,
};

/// ASCII backspace.
const BS: u8 = 8;

fn main_imp() -> io::Result<()> {
    let mut stdin = io::stdin().bytes();
    let mut stdout = io::stdout();
    let mut putch = |b| stdout.write_all(&[b]);
    let mut getch = || stdin.next().unwrap_or_else(|| exit(0));

    let mut a = getch()?;
    putch(a)?;

    // Ugh, the pattern is apparently: c ESC [0m c

    loop {
        let b = getch()?;
        if b == BS {
            let c = getch()?;
            if c == 27 {
                continue;
            }
            eprintln!("DEBUG: {a} {c}");
            putch(b)?;
            putch(c)?;
            a = c;
        } else {
            putch(b)?;
            a = b;
        }
    }
}

fn main() {
    if let Err(err) = main_imp() {
        eprintln!("error: {err}");
        exit(1);
    }
}
