//! Mutation fuzzing: every `.cp` file in `std/`, the corpus `spec.cp`
//! files and the lint records, with tokens deleted, doubled, swapped,
//! replaced or cut off, must never crash `cpl lint` or `cpl fmt`, and
//! whatever fmt writes it leaves alone the second time
//! (`handbook/guidelines/lints-checklist.md`, `lints/R22` and `lints/R23`).
//! The mutations come from a fixed seed, so a failure reproduces.

use super::*;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};

/// Mutants per file; `CPL_FUZZ_MUTANTS` asks for a deeper run.
fn mutants_per_file() -> u64 {
    std::env::var("CPL_FUZZ_MUTANTS")
        .ok()
        .and_then(|n| n.parse().ok())
        .unwrap_or(3)
}

fn cp_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "cp") {
            out.push(path);
        }
    }
}

fn sources(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    cp_files(&root.join(crate::STD_DIR), &mut out);
    for entry in std::fs::read_dir(root.join("corpus/problems")).unwrap() {
        let spec = entry.unwrap().path().join("spec.cp");
        if spec.is_file() {
            out.push(spec);
        }
    }
    for entry in std::fs::read_dir(root.join("spec/lints")).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            cp_files(&path, &mut out);
        }
    }
    out.sort();
    out
}

/// A small deterministic generator (SplitMix64).
struct Rng(u64);

impl Rng {
    fn below(&mut self, n: usize) -> usize {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        ((z ^ (z >> 31)) % n.max(1) as u64) as usize
    }
}

/// `src` with one token-level edit, and what the edit was.
fn mutate(src: &str, rng: &mut Rng) -> (String, String) {
    let spans: Vec<(usize, usize)> = cpl_syntax::lex(src)
        .tokens
        .iter()
        .map(|t| (t.span.start as usize, t.span.end as usize))
        .filter(|(s, e)| e > s)
        .collect();
    if spans.len() < 2 {
        return (src.to_string(), "none".into());
    }
    let i = rng.below(spans.len() - 1);
    let (s, e) = spans[i];
    let (s2, e2) = spans[i + 1];
    let tok = &src[s..e];
    match rng.below(5) {
        0 => (
            format!("{}{}", &src[..s], &src[e..]),
            format!("delete `{tok}` at {s}"),
        ),
        1 => (
            format!("{}{tok} {}", &src[..s], &src[s..]),
            format!("double `{tok}` at {s}"),
        ),
        2 => (
            format!(
                "{}{}{}{tok}{}",
                &src[..s],
                &src[s2..e2],
                &src[e..s2],
                &src[e2..]
            ),
            format!("swap `{tok}` and `{}` at {s}", &src[s2..e2]),
        ),
        3 => {
            let (rs, re) = spans[rng.below(spans.len())];
            let other = &src[rs..re];
            (
                format!("{}{other}{}", &src[..s], &src[e..]),
                format!("replace `{tok}` at {s} with `{other}`"),
            )
        }
        _ => (src[..e].to_string(), format!("cut after `{tok}` at {s}")),
    }
}

/// What goes wrong on `src`, if anything.
fn trouble(src: &str, config: &Config) -> Option<String> {
    if let Err(panic) = catch_unwind(AssertUnwindSafe(|| lint(src, config))) {
        return Some(format!("lint panics: {}", message(&panic)));
    }
    match catch_unwind(|| cpl_fmt::format(src)) {
        Err(panic) => Some(format!("fmt panics: {}", message(&panic))),
        Ok(Ok(out)) => match catch_unwind(|| cpl_fmt::format(&out)) {
            Ok(Ok(again)) if again == out => None,
            Ok(Ok(_)) => Some("fmt changes its own output".into()),
            Ok(Err(err)) => Some(format!("fmt rejects its own output: {err:?}")),
            Err(panic) => Some(format!("fmt panics on its own output: {}", message(&panic))),
        },
        Ok(Err(_)) => None,
    }
}

fn message(panic: &Box<dyn std::any::Any + Send>) -> String {
    let text = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("?");
    text.lines().next().unwrap_or("").to_string()
}

/// With `CPL_FUZZ_DUMP=<dir>`, a failing mutant is written there to reproduce.
fn dump(name: &str, n: usize, mutant: &str) {
    if let Some(dir) = std::env::var_os("CPL_FUZZ_DUMP") {
        let file = format!("{}-{n}.cp", name.replace('/', "_").trim_end_matches(".cp"));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(Path::new(&dir).join(file), mutant).unwrap();
    }
}

#[test]
fn mutated_sources_never_crash_lint_or_fmt() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let config = tests::all_on();
    let files = sources(&root);
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let per_file = mutants_per_file();
    let mut wrong: Vec<String> = std::thread::scope(|s| {
        let workers: Vec<_> = (0..threads)
            .map(|w| {
                let (root, config, files) = (&root, &config, &files);
                s.spawn(move || {
                    let mut wrong = Vec::new();
                    for (n, path) in files.iter().enumerate().skip(w).step_by(threads) {
                        let src = std::fs::read_to_string(path).unwrap();
                        let name = path
                            .strip_prefix(root)
                            .unwrap_or(path)
                            .display()
                            .to_string();
                        if let Some(why) = trouble(&src, config) {
                            wrong.push(format!("{name}: {why}"));
                        }
                        let mut rng = Rng(n as u64);
                        for _ in 0..per_file {
                            let (mutant, edit) = mutate(&src, &mut rng);
                            if let Some(why) = trouble(&mutant, config) {
                                dump(&name, wrong.len(), &mutant);
                                wrong.push(format!("{name}, {edit}: {why}"));
                            }
                        }
                    }
                    wrong
                })
            })
            .collect();
        workers
            .into_iter()
            .flat_map(|w| w.join().unwrap())
            .collect()
    });
    wrong.sort();
    assert!(
        wrong.is_empty(),
        "{} failures:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}
