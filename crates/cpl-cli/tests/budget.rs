//! The budgets `cpl` holds itself to, measured on the binary a user runs:
//! `cpl run` answers in under 200 ms, `cpl build` in under a second, and
//! the C++ it writes is under the 64 KB that judges accept.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

#[path = "support/shards.rs"]
mod shards;

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

/// The release `cpl`, built (or brought up to date) on first use: `cargo
/// test` builds only the dev profile, and timing that measures the build
/// profile instead of what a user runs.
fn cpl() -> &'static Path {
    static CPL: OnceLock<PathBuf> = OnceLock::new();
    CPL.get_or_init(|| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let target = std::env::var_os("CARGO_TARGET_DIR")
            .map_or_else(|| root.join("target"), |dir| root.join(dir));
        let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
        // Cargo hands this test the `CPL_CHANNEL` cpl-cli's build script
        // sets; passed back to cargo, it changes the build's inputs, and
        // each build undoes the one before.
        let status = Command::new(cargo)
            .args(["build", "--release", "-p", "cpl-cli", "--bin", "cpl"])
            .env_remove("CPL_CHANNEL")
            .current_dir(&root)
            .status()
            .expect("cargo runs");
        assert!(
            status.success(),
            "`cargo build --release -p cpl-cli` failed"
        );
        target
            .join("release")
            .join(format!("cpl{}", std::env::consts::EXE_SUFFIX))
    })
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
    let out = Command::new(cpl())
        .args(args)
        .arg(program)
        .stdin(stdin)
        .output()
        .expect("cpl runs");
    (start.elapsed(), out.stdout)
}

/// The first try within `budget`, else the fastest of five: a busy machine
/// slows one run, not all, and a program already within budget needs no
/// retry (197 programs at five tries each took 78 s).
fn best(args: &[&str], program: &Path, budget: Duration) -> (Duration, Vec<u8>) {
    let mut fastest = timed(args, program);
    for _ in 1..5 {
        if fastest.0 < budget {
            break;
        }
        let next = timed(args, program);
        if next.0 < fastest.0 {
            fastest = next;
        }
    }
    fastest
}

const SHARDS: usize = 8;

/// The name a program shards by.
fn key(program: &Path) -> String {
    program
        .file_name()
        .map_or_else(String::new, |n| n.to_string_lossy().into_owned())
}

fn shard(k: usize) -> Vec<PathBuf> {
    shards::mine(programs(), |p| key(p), k, SHARDS)
}

fn keys() -> Vec<String> {
    programs().iter().map(|p| key(p)).collect()
}

fn run_answers_within_200_ms(k: usize) {
    for program in shard(k) {
        let (took, _) = best(&["run"], &program, Duration::from_millis(200));
        assert!(
            took < Duration::from_millis(200),
            "`cpl run {}` took {took:?}",
            program.display()
        );
    }
}

fn build_takes_under_a_second_and_fits_64_kb(k: usize) {
    for program in shard(k) {
        let (took, cpp) = best(&["build", "-o", "-"], &program, Duration::from_secs(1));
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

shards::shard_tests!(
    run_answers_within_200_ms,
    keys(),
    the_run_shards_cover_every_program,
    [
        run_answers_within_200_ms_0 = 0,
        run_answers_within_200_ms_1 = 1,
        run_answers_within_200_ms_2 = 2,
        run_answers_within_200_ms_3 = 3,
        run_answers_within_200_ms_4 = 4,
        run_answers_within_200_ms_5 = 5,
        run_answers_within_200_ms_6 = 6,
        run_answers_within_200_ms_7 = 7,
    ]
);

shards::shard_tests!(
    build_takes_under_a_second_and_fits_64_kb,
    keys(),
    the_build_shards_cover_every_program,
    [
        build_takes_under_a_second_and_fits_64_kb_0 = 0,
        build_takes_under_a_second_and_fits_64_kb_1 = 1,
        build_takes_under_a_second_and_fits_64_kb_2 = 2,
        build_takes_under_a_second_and_fits_64_kb_3 = 3,
        build_takes_under_a_second_and_fits_64_kb_4 = 4,
        build_takes_under_a_second_and_fits_64_kb_5 = 5,
        build_takes_under_a_second_and_fits_64_kb_6 = 6,
        build_takes_under_a_second_and_fits_64_kb_7 = 7,
    ]
);
