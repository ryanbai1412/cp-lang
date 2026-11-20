//! The budgets `cpl` holds itself to, measured on the binary a user runs:
//! `cpl run` answers in under 200 ms, `cpl build` in under a second, and
//! the C++ it writes is under the 64 KB that judges accept.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

fn programs() -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/programs");
    let mut found: Vec<PathBuf> = std::fs::read_dir(dir)
        .expect("tests/programs exists")
        .map(|entry| entry.expect("readable entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "cp"))
        .collect();
    found.sort();
    found
}

/// Runs `cpl` with `args`, feeding it the program's `.in` file, and gives
/// back how long it took and what it printed.
fn timed(args: &[&str], program: &Path) -> (Duration, Vec<u8>) {
    let input = program.with_extension("in");
    let stdin = match std::fs::File::open(input) {
        Ok(file) => Stdio::from(file),
        Err(_) => Stdio::null(),
    };
    let start = Instant::now();
    let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args(args)
        .arg(program)
        .stdin(stdin)
        .output()
        .expect("cpl runs");
    (start.elapsed(), out.stdout)
}

/// The fastest of a few tries: a busy machine slows one run, not all.
fn best(args: &[&str], program: &Path) -> (Duration, Vec<u8>) {
    (0..5)
        .map(|_| timed(args, program))
        .min_by_key(|(took, _)| *took)
        .expect("at least one try")
}

#[test]
fn run_answers_within_200_ms() {
    for program in programs() {
        let (took, _) = best(&["run"], &program);
        assert!(
            took < Duration::from_millis(200),
            "`cpl run {}` took {took:?}",
            program.display()
        );
    }
}

#[test]
fn build_takes_under_a_second_and_fits_64_kb() {
    for program in programs() {
        let (took, cpp) = best(&["build", "-o", "-"], &program);
        assert!(
            took < Duration::from_secs(1),
            "`cpl build {}` took {took:?}",
            program.display()
        );
        assert!(
            !cpp.is_empty(),
            "`cpl build {}` wrote nothing",
            program.display()
        );
        assert!(
            cpp.len() < 64 * 1024,
            "the C++ for {} is {} bytes",
            program.display(),
            cpp.len()
        );
    }
}
