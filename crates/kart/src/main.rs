use std::collections::HashMap;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

// ── Subcommand discovery ────────────────────────────────────────

fn command_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(home) = env::var_os("HOME") {
        dirs.push(PathBuf::from(home).join(".kart/commands"));
    }
    if let Ok(exe) = env::current_exe() {
        if let Some(dir) = exe.parent() {
            dirs.push(dir.join("../libexec/kart"));
        }
    }
    dirs
}

fn is_executable(path: &PathBuf) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        path.is_file()
            && fs::metadata(path)
                .map(|m| m.permissions().mode() & 0o111 != 0)
                .unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        path.is_file()
    }
}

/// Discover available subcommands by scanning command dirs.
/// Returns name -> path mapping.
fn discover_commands() -> HashMap<String, PathBuf> {
    let mut commands = HashMap::new();
    for dir in command_dirs() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !is_executable(&path) {
                continue;
            }
            if let Some(name) = path.file_name().and_then(OsStr::to_str) {
                let name = name.strip_prefix("kart-").unwrap_or(name);
                commands.entry(name.to_string()).or_insert(path);
            }
        }
    }
    commands
}

// ── Completion protocol ─────────────────────────────────────────

/// Ask a subcommand for completions.
///
/// Calls: `<subcmd_path> --complete -- <args...>`
///
/// Expects one completion per line, with an optional tab-separated
/// description.
fn get_completions_from(subcmd: &PathBuf, args: &[String]) -> Vec<(String, Option<String>)> {
    let output = Command::new(subcmd)
        .arg("--complete")
        .arg("--")
        .args(args)
        .output();

    let Ok(output) = output else { return vec![] };
    if !output.status.success() {
        return vec![];
    }

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| !l.is_empty())
        .map(|line| match line.split_once('\t') {
            Some((val, desc)) => (val.to_string(), Some(desc.to_string())),
            None => (line.to_string(), None),
        })
        .collect()
}

/// Handle `kart --complete -- <args...>`.
fn complete(args: &[String]) {
    let commands = discover_commands();

    if args.is_empty() || (args.len() == 1 && !args[0].starts_with('-')) {
        // Completing the subcommand name itself
        let partial = args.first().map(String::as_str).unwrap_or("");
        for name in commands.keys() {
            if name.starts_with(partial) {
                println!("{name}");
            }
        }
        return;
    }

    // Delegate to the subcommand
    let subcmd_name = &args[0];
    if let Some(subcmd_path) = commands.get(subcmd_name) {
        let completions = get_completions_from(subcmd_path, &args[1..]);
        for (val, desc) in completions {
            match desc {
                Some(d) => println!("{val}\t{d}"),
                None => println!("{val}"),
            }
        }
    }
}

// ── Shell script generation ─────────────────────────────────────

fn generate_zsh() {
    print!(
        r#"#compdef kart

_kart() {{
    local -a completions
    local line
    local IFS=$'\n'

    completions=($(kart --complete -- "${{words[@]:1}}" 2>/dev/null))

    local -a vals
    for line in "${{completions[@]}}"; do
        if [[ "$line" == *$'\t'* ]]; then
            vals+=("${{line%%$'\t'*}}:${{line#*$'\t'}}")
        else
            vals+=("$line")
        fi
    done

    if (( $#vals )); then
        _describe 'command' vals
    fi
}}

_kart "$@"
"#
    );
}

fn generate_bash() {
    print!(
        r#"_kart_completions() {{
    local IFS=$'\n'
    local completions
    completions=($(kart --complete -- "${{COMP_WORDS[@]:1}}" 2>/dev/null))

    local -a vals
    for line in "${{completions[@]}}"; do
        vals+=("${{line%%$'\t'*}}")
    done

    COMPREPLY=($(compgen -W "${{vals[*]}}" -- "${{COMP_WORDS[COMP_CWORD]}}"))
}}

complete -F _kart_completions kart
"#
    );
}

fn generate_xonsh() {
    print!(
        r#"import subprocess


def _kart_completer(prefix, line, begidx, endidx, ctx):
    args = line.split()[1:]
    try:
        result = subprocess.run(
            ["kart", "--complete", "--"] + args,
            capture_output=True,
            text=True,
            timeout=2,
        )
    except Exception:
        return set()
    candidates = set()
    for line in result.stdout.splitlines():
        val = line.split("\t")[0]
        if val:
            candidates.add(val)
    return candidates


completer remove kart 2>/dev/null
completer add kart _kart_completer end
"#
    );
}

fn generate_completions(shell: &str) {
    match shell {
        "zsh" => generate_zsh(),
        "bash" => generate_bash(),
        "xonsh" => generate_xonsh(),
        other => eprintln!("kart: unknown shell '{other}'"),
    }
}

// ── Main dispatch ───────────────────────────────────────────────

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    match args.first().map(String::as_str) {
        Some("completions") => {
            let shell = args.get(1).map(String::as_str).unwrap_or("zsh");
            generate_completions(shell);
        }
        Some("--complete") => {
            let rest: Vec<String> = args
                .into_iter()
                .skip(1)
                .skip_while(|a| a == "--")
                .collect();
            complete(&rest);
        }
        Some(subcmd) => {
            let commands = discover_commands();
            if let Some(path) = commands.get(subcmd) {
                let status = Command::new(path)
                    .args(&args[1..])
                    .status()
                    .expect("failed to execute subcommand");
                std::process::exit(status.code().unwrap_or(1));
            }
            eprintln!("kart: unknown command '{subcmd}'");
            std::process::exit(1);
        }
        None => {
            let commands = discover_commands();
            println!("available commands:");
            let mut names: Vec<&str> = commands.keys().map(String::as_str).collect();
            names.sort();
            for name in names {
                println!("  {name}");
            }
        }
    }
}
