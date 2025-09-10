use rk_lev::lev;

fn print_lev(a: &str, b: &str) {
    println!("{:4} {b}", lev(a.chars(), b.chars()));
}

fn main() {
    let mut args = std::env::args();
    let (Some(a), Some(b)) = (args.nth(1), args.next()) else {
        eprintln!("Usage: lev STRING1 STRING2 [STRINGS...]");
        std::process::exit(2);
    };
    print_lev(&a, &b);
    for b in args {
        print_lev(&a, &b);
    }
}
