//! `cpl-prover FILE.cp ...`: proves the `@law`s each file states.
//!
//! Prints one verdict per law and exits with 1 when a law is broken. A
//! law the prover can't decide is reported and does not fail: `cpl test`
//! tries it on sample values.

use std::process::ExitCode;

fn main() -> ExitCode {
    let paths: Vec<String> = std::env::args().skip(1).collect();
    if paths.is_empty() || paths.iter().any(|path| path == "-h" || path == "--help") {
        eprintln!("usage: cpl-prover FILE.cp ...\n\nProves the `@law`s of each file, or shows the values that break one.");
        return ExitCode::from(2);
    }
    let mut broken = false;
    for path in &paths {
        let text = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(error) => {
                eprintln!("cpl-prover: can't read `{path}`: {error}");
                return ExitCode::from(2);
            }
        };
        let report = cpl_prover::laws::check_source(&text);
        if report.results.is_empty() {
            println!("{path}: no `@law` to prove");
            continue;
        }
        println!("{path}:");
        for line in report.to_string().lines() {
            println!("  {line}");
        }
        broken |= report.results.iter().any(|result| result.verdict.counterexample().is_some());
    }
    if broken {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
