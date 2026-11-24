//! `cpl run` fits the memory a judge gives (compiler-changes/R14): on the
//! largest input of each of the corpus problems with the largest inputs,
//! the interpreter's peak memory stays under the problem's limit (its
//! `// limits:` line, else its judge's).

use cpl_stress::program::Program;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// How many problems, largest input first, the gate runs.
const PROBLEMS: usize = 6;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

/// Each problem's largest test input, for the problems with the largest,
/// making every package's cases first so the choice never depends on a cache.
fn largest_inputs(root: &Path) -> Vec<(String, PathBuf)> {
    let specs = cpl_corpus::files(root, "spec.cp");
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get());
    std::thread::scope(|s| {
        for chunk in specs.chunks(specs.len().div_ceil(threads).max(1)) {
            s.spawn(move || {
                for spec in chunk {
                    let package = spec.parent().unwrap();
                    let out = cpl_contest::package::cases_dir(package);
                    if let Err(why) = cpl_contest::package::make(package, &out) {
                        panic!("making {}'s cases failed: {why}", package.display());
                    }
                }
            });
        }
    });
    let mut found = Vec::new();
    for spec in &specs {
        let dir = cpl_contest::package::cases_dir(spec.parent().unwrap());
        let tests = std::fs::read_dir(&dir)
            .unwrap_or_else(|e| panic!("{}: {e}, after making its cases", dir.display()));
        let largest = tests
            .flatten()
            .map(|t| t.path())
            .filter(|p| p.extension().is_some_and(|e| e == "in"))
            .filter_map(|p| Some((std::fs::metadata(&p).ok()?.len(), p)))
            .max_by(|a, b| a.0.cmp(&b.0).then_with(|| b.1.cmp(&a.1)));
        if let Some((size, input)) = largest {
            let name = spec
                .parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned();
            found.push((size, name, input));
        }
    }
    found.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    found
        .into_iter()
        .take(PROBLEMS)
        .map(|(_, name, input)| (name, input))
        .collect()
}

/// Runs `cpl run spec < input` under the stress runner, the one place that
/// samples a child's high-water mark, and returns its peak memory.
fn peak_kb(spec: &Path, input: &Path) -> u64 {
    let cpl = env!("CARGO_BIN_EXE_cpl");
    let spec_arg = spec.display().to_string();
    let input = std::fs::read_to_string(input).expect("read input");
    let out = Program::command("cpl", &[cpl, "run", &spec_arg])
        .run(&input, Duration::from_secs(300))
        .expect("spawn cpl");
    assert!(
        out.ok(),
        "`cpl run {}` failed ({:?}): {}",
        spec.display(),
        out.exit,
        out.stderr
    );
    out.peak_kb.expect("Linux reports a high-water mark")
}

/// The run on the `rank`th largest input, one test each so nextest spreads
/// them over CI's test partitions.
#[cfg(target_os = "linux")]
fn largest_input_fits_the_judges_memory(rank: usize) {
    let root = repo_root();
    let inputs = largest_inputs(&root);
    assert_eq!(inputs.len(), PROBLEMS, "fewer corpus problems with inputs");
    let (name, input) = &inputs[rank];
    let spec = cpl_corpus::package(&root, name).join("spec.cp");
    let src = std::fs::read_to_string(&spec).expect("read spec");
    let limit = cpl_contest::problem::Limits::of(&spec, &src).memory_kb;
    let peak = peak_kb(&spec, input);
    assert!(
        peak <= limit,
        "{name}: `cpl run` peaks at {} MB on {}, over the {} MB limit",
        peak / 1024,
        input.display(),
        limit / 1024
    );
}

#[test]
#[cfg(target_os = "linux")]
fn the_interpreter_runs_the_largest_corpus_inputs_in_the_judges_memory_0() {
    largest_input_fits_the_judges_memory(0);
}

#[test]
#[cfg(target_os = "linux")]
fn the_interpreter_runs_the_largest_corpus_inputs_in_the_judges_memory_1() {
    largest_input_fits_the_judges_memory(1);
}

#[test]
#[cfg(target_os = "linux")]
fn the_interpreter_runs_the_largest_corpus_inputs_in_the_judges_memory_2() {
    largest_input_fits_the_judges_memory(2);
}

#[test]
#[cfg(target_os = "linux")]
fn the_interpreter_runs_the_largest_corpus_inputs_in_the_judges_memory_3() {
    largest_input_fits_the_judges_memory(3);
}

#[test]
#[cfg(target_os = "linux")]
fn the_interpreter_runs_the_largest_corpus_inputs_in_the_judges_memory_4() {
    largest_input_fits_the_judges_memory(4);
}

#[test]
#[cfg(target_os = "linux")]
fn the_interpreter_runs_the_largest_corpus_inputs_in_the_judges_memory_5() {
    largest_input_fits_the_judges_memory(5);
}
