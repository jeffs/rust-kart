//! CLI entry point for dgmod

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

use dgmod::workspace::detect_workspace;

/// Analyze Rust module dependencies and generate Mermaid diagrams
#[derive(Parser)]
#[command(name = "dgmod")]
#[command(about = "Generate Mermaid diagrams of Rust module dependencies")]
struct Args {
    /// Path to the Rust crate or workspace to analyze
    #[arg(default_value = ".")]
    path: PathBuf,
}

fn main() -> ExitCode {
    let args = Args::parse();

    // Try to detect workspace first
    match detect_workspace(&args.path) {
        Ok(members) if members.len() > 1 => {
            // Workspace with multiple members - analyze each
            for member in members {
                match dgmod::analyze_crate(&member.path, &member.name) {
                    Ok(graph) => {
                        println!("## {}\n", graph.crate_name);
                        println!("```mermaid");
                        print!("{}", graph.to_mermaid());
                        println!("```\n");
                    }
                    Err(e) => {
                        eprintln!("error: {e}");
                        return ExitCode::FAILURE;
                    }
                }
            }
            ExitCode::SUCCESS
        }
        Ok(members) if members.len() == 1 => {
            // Single-member workspace - analyze it
            let member = &members[0];
            analyze_single_crate(&member.path, &member.name)
        }
        Ok(_) | Err(_) => {
            // Not a workspace or failed to detect - try as single crate
            let crate_name = args
                .path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("crate");
            analyze_single_crate(&args.path, crate_name)
        }
    }
}

fn analyze_single_crate(path: &std::path::Path, name: &str) -> ExitCode {
    match dgmod::analyze_crate(path, name) {
        Ok(graph) => {
            println!("## {}\n", graph.crate_name);
            println!("```mermaid");
            print!("{}", graph.to_mermaid());
            println!("```");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
