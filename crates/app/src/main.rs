//! `schemify`: CLI entry point and the spines (ARCHITECTURE.md § Spines).
//!
//! Exit codes (D27): 0 success, 1 design errors (violations / merge conflicts),
//! 2 usage or I/O errors.

use std::process::ExitCode;

const USAGE: &str = "\
usage: schemify <command> [args]

commands:
  import <xschem-dir> <out-dir>        convert an xschem library (designs or PDK devices)
  netlist <cell> [--dialect D] [--config C] [--corner K] [-o FILE] [--allow-errors]
  check <cell>                         report violations
  fmt <file>...                        rewrite files in canonical form
  merge-driver <O> <A> <B> [<path>]    git merge driver (writes result to A)
  diff <A> <B>                         semantic diff of two files
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(command) = args.first() else {
        eprint!("{USAGE}");
        return ExitCode::from(2);
    };
    // Spine selection is the only branch on which work happens (codebase-clean-code rule 1).
    match command.as_str() {
        "import" | "netlist" | "check" | "fmt" | "merge-driver" | "diff" => {
            eprintln!("schemify {command}: not implemented yet");
            ExitCode::from(2)
        }
        "-h" | "--help" | "help" => {
            print!("{USAGE}");
            ExitCode::SUCCESS
        }
        _ => {
            eprint!("unknown command `{command}`\n{USAGE}");
            ExitCode::from(2)
        }
    }
}
