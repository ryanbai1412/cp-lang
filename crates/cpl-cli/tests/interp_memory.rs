//! `cpl run` fits the memory a judge gives (compiler-changes/R14): on the
//! largest input of each of the corpus problems with the largest inputs,
//! the interpreter's peak memory stays under the problem's limit (its
//! `// limits:` line, else its judge's).

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
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
        let dir = problem.path();
        let Ok(tests) = std::fs::read_dir(dir.join("tests")) else {
            continue;
        };
        if !dir.join("spec.cp").exists() {
            continue;
        }
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

/// The high-water mark of a running process, in kilobytes.
fn high_water_kb(pid: u32) -> Option<u64> {
    let status = std::fs::read_to_string(format!("/proc/{pid}/status")).ok()?;
    let line = status.lines().find(|l| l.starts_with("VmHWM:"))?;
    line.split_whitespace().nth(1)?.parse().ok()
}

/// Runs `cpl run spec < input` and returns its peak memory, sampled until it
/// exits, so a lower bound on the true peak.
fn peak_kb(spec: &Path, input: &Path) -> u64 {
    let mut child = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg("run")
        .arg(spec)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn cpl");
    let bytes = std::fs::read(input).expect("read input");
    let mut stdin = child.stdin.take().unwrap();
    let writer = std::thread::spawn(move || stdin.write_all(&bytes));
    let mut peak = 0;
    while child.try_wait().expect("wait cpl").is_none() {
        peak = peak.max(high_water_kb(child.id()).unwrap_or(0));
        std::thread::sleep(Duration::from_millis(2));
    }
    let _ = writer.join();
    let out = child.wait_with_output().expect("wait cpl");
    assert!(
        out.status.success(),
        "`cpl run {}` failed: {}",
        spec.display(),
        String::from_utf8_lossy(&out.stderr)
    );
    peak
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
