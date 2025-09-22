use std::os::unix::process::CommandExt;

fn die(what: impl std::fmt::Display) -> ! {
    eprintln!("Error: {what}");
    std::process::exit(1)
}

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(command) = args.next() else {
        die("expected command");
    };
    die(std::process::Command::new(format!("rk-{command}"))
        .args(args)
        .exec());
}
