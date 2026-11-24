//! `cpl run` fits the memory a judge gives (compiler-changes/R14): on the
//! largest input of each of the corpus problems with the largest inputs,
//! the interpreter's peak memory stays under the problem's limit (its
//! `// limits:` line, else its judge's).

use cpl_stress::program::Program;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// How many problems, largest input first, the gate runs.
const PROBLEMS: usize = 6;

/// Problems whose run is over the limit today, with the cause. Delete an
/// entry when its run fits; the test fails while a listed run fits.
const KNOWN_OVER_MEMORY: &[(&str, &str)] = &[
    (
        "btoi17p6",
        "a 200,000-deep recursion: about 6 KB of interpreter stack a call",
    ),
    ("egoi23p7", "about 300 MB peak"),
    ("sdoi17r2p4", "about 290 MB peak"),
];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

/// Each problem's largest test input, for the problems with the largest.
fn largest_inputs(root: &Path) -> Vec<(String, PathBuf)> {
    let mut found = Vec::new();
    let problems = std::fs::read_dir(root.join("corpus/problems")).expect("corpus/problems");
    for problem in problems.flatten() {
        let spec = problem.path().join("spec.cp");
        if !spec.exists() {
            continue;
        }
        // tests.toml cases are what `scripts/corpus.py make` wrote under target/.
        let Ok(tests) = std::fs::read_dir(cpl_contest::problem::find_tests_dir(&spec)) else {
            continue;
        };
        let largest = tests
            .flatten()
            .map(|t| t.path())
            .filter(|p| p.extension().is_some_and(|e| e == "in"))
            .filter_map(|p| Some((std::fs::metadata(&p).ok()?.len(), p)))
            .max_by(|a, b| a.0.cmp(&b.0).then_with(|| b.1.cmp(&a.1)));
        if let Some((size, input)) = largest {
            let name = problem.file_name().to_string_lossy().into_owned();
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
    let spec = root.join("corpus/problems").join(name).join("spec.cp");
    let src = std::fs::read_to_string(&spec).expect("read spec");
    let limit = cpl_contest::problem::Limits::of(&spec, &src).memory_kb;
    let peak = peak_kb(&spec, input);
    let known = KNOWN_OVER_MEMORY.iter().any(|(k, _)| k == name);
    match (peak > limit, known) {
        (true, false) => panic!(
            "{name}: `cpl run` peaks at {} MB on {}, over the {} MB limit",
            peak / 1024,
            input.display(),
            limit / 1024
        ),
        (false, true) => panic!(
            "{name}: peaks at {} MB, under the {} MB limit now; delete it from KNOWN_OVER_MEMORY",
            peak / 1024,
            limit / 1024
        ),
        _ => {}
    }
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
