//! No source text crashes the front end, and every source gets the same
//! diagnostics from a warm database as from a fresh one. Every corpus `spec.cp` is
//! mutated with a fixed seed (truncated, a span cut out, a token or line
//! inserted, a line duplicated), and each copy goes through the checker,
//! lowering and the linter. A panic on any copy fails the test and prints
//! the seed and the mutated text, so the failure reproduces exactly.
//!
//! `CPL_FUZZ_ROUNDS=200` runs more copies per file than the default.
//!
//! Rule: `errors/R12`, handbook/guidelines/errors-checklist.md#r12-every-tool-shows-the-same-diagnostic.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }
}

const TOKENS: &[&str] = &[
    "(",
    ")",
    "[",
    "]",
    "{",
    "}",
    ":",
    "::",
    ",",
    ".",
    "..",
    "..=",
    "=",
    "==",
    "=>",
    "->",
    "?",
    "!",
    "??",
    "\"",
    "'",
    "`",
    "\\",
    "#",
    "@",
    "mut ",
    "let ",
    "var ",
    "fn ",
    "for ",
    "in ",
    "if ",
    "else",
    "match ",
    "return ",
    "lends ",
    "domain ",
    "struct ",
    "enum ",
    "use ",
    "with ",
    "0",
    "-1",
    "9223372036854775807",
    "1e9",
    "_",
    "x",
    "Int",
    "?Int",
    "[V]Int",
    "\n",
    "\n    ",
    "    ",
    "\t",
    "é",
    "//",
    "/*",
    "<",
    ">",
    "<<",
    "**",
    "%",
    "*",
    "+",
    "-",
];

/// A char boundary at or after `at`.
fn boundary(s: &str, mut at: usize) -> usize {
    at = at.min(s.len());
    while !s.is_char_boundary(at) {
        at += 1;
    }
    at
}

fn mutate(src: &str, rng: &mut Rng) -> String {
    let at = boundary(src, rng.below(src.len() + 1));
    match rng.below(5) {
        0 => src[..at].to_string(),
        1 => {
            let end = boundary(src, at + 1 + rng.below(40));
            format!("{}{}", &src[..at], &src[end..])
        }
        2 => {
            let tok = TOKENS[rng.below(TOKENS.len())];
            format!("{}{tok}{}", &src[..at], &src[at..])
        }
        3 => {
            let lines: Vec<&str> = src.lines().collect();
            let i = rng.below(lines.len());
            let j = rng.below(lines.len() + 1);
            let mut out = lines.clone();
            out.insert(j, lines.get(i).copied().unwrap_or(""));
            out.join("\n")
        }
        _ => {
            let a = boundary(src, rng.below(src.len() + 1));
            let b = boundary(src, a + rng.below(20));
            let piece = &src[a..b];
            format!("{}{piece}{}", &src[..at], &src[at..])
        }
    }
}

/// Every diagnostic the checker, lowering and linter report, as text.
fn front_end(db: &mut cpl_check::Session, path: &str, src: &str) -> Vec<String> {
    db.set_file(path, src);
    let analysis = db.analyze(path).expect("the file was just set");
    let lints = cpl_lint::lint(src, &cpl_lint::Config::default());
    cpl_ir::check(&analysis)
        .iter()
        .chain(&lints)
        .map(|d| {
            format!(
                "{:?} {}..{} {}",
                d.code, d.span.start, d.span.end, d.message
            )
        })
        .collect()
}

#[test]
fn mutated_programs_never_crash_and_report_deterministically() {
    let rounds: usize = std::env::var("CPL_FUZZ_ROUNDS")
        .ok()
        .and_then(|r| r.parse().ok())
        .unwrap_or(12);
    let mut paths: Vec<PathBuf> = std::fs::read_dir(root().join("corpus/problems"))
        .unwrap()
        .flatten()
        .map(|e| e.path().join("spec.cp"))
        .filter(|p| p.exists())
        .collect();
    paths.sort();
    std::panic::set_hook(Box::new(|_| {}));
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let runs = paths.len() * rounds;
    let crashes: Vec<String> = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..threads)
            .map(|t| {
                let paths = &paths;
                scope.spawn(move || {
                    let mut crashes = Vec::new();
                    let mut db = cpl_check::Session::new();
                    for (k, path) in paths.iter().enumerate().skip(t).step_by(threads) {
                        let src = std::fs::read_to_string(path).unwrap();
                        let name = path.to_string_lossy().into_owned();
                        for round in 0..rounds {
                            let seed = 0x9E37_79B9_7F4A_7C15 ^ ((k as u64) << 32 | round as u64);
                            let mut rng = Rng(seed | 1);
                            let mut copy = mutate(&src, &mut rng);
                            if rng.below(3) == 0 {
                                copy = mutate(&copy, &mut rng);
                            }
                            let ok =
                                catch_unwind(AssertUnwindSafe(|| front_end(&mut db, &name, &copy)));
                            match &ok {
                                Ok(warm) if round % 4 == 0 => {
                                    let fresh =
                                        front_end(&mut cpl_check::Session::new(), &name, &copy);
                                    if *warm != fresh {
                                        crashes.push(format!(
                                            "{name} seed {seed:#x}: a warm database reports\n{}\nand a fresh one\n{}\n{copy}",
                                            warm.join("\n"),
                                            fresh.join("\n")
                                        ));
                                    }
                                }
                                _ => {}
                            }
                            if let Err(e) = ok {
                                let msg = e
                                    .downcast_ref::<String>()
                                    .cloned()
                                    .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
                                    .unwrap_or_default();
                                crashes.push(format!("{name} seed {seed:#x}: {msg}\n{copy}"));
                                db = cpl_check::Session::new();
                            }
                        }
                    }
                    crashes
                })
            })
            .collect();
        workers
            .into_iter()
            .flat_map(|w| w.join().unwrap())
            .collect()
    });
    let _ = std::panic::take_hook();
    println!("{runs} mutated programs, {} failures", crashes.len());
    assert!(
        crashes.is_empty(),
        "{} of {runs} mutated programs crash or differ; first:\n{}",
        crashes.len(),
        crashes
            .iter()
            .take(3)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n-----\n")
    );
}

/// Diagnostics add under 10% to the front end: every corpus spec, broken by
/// a mutation that keeps its size (a token, a line or a piece pasted in) so
/// that it reports something, goes through the checker, lowering and linter
/// against a warm database, alternating with its clean original. Summed over
/// the corpus, the broken copies take at most 1.10x the originals' time.
/// Rule: `errors/R17`, handbook/guidelines/errors-checklist.md#r17-a-mistake-costs-no-more-time-than-a-clean-file.
#[test]
fn a_broken_copy_checks_within_10_percent_of_its_original() {
    use std::time::{Duration, Instant};
    let mut paths: Vec<PathBuf> = std::fs::read_dir(root().join("corpus/problems"))
        .unwrap()
        .flatten()
        .map(|e| e.path().join("spec.cp"))
        .filter(|p| p.exists())
        .collect();
    paths.sort();
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let (clean, broken, copies) = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..threads)
            .map(|t| {
                let paths = &paths;
                scope.spawn(move || {
                    let mut db = cpl_check::Session::new();
                    let (mut clean, mut broken, mut copies) = (Duration::ZERO, Duration::ZERO, 0);
                    for (k, path) in paths.iter().enumerate().skip(t).step_by(threads) {
                        let src = std::fs::read_to_string(path).unwrap();
                        let name = path.to_string_lossy().into_owned();
                        let mut rng = Rng((0x5EED_u64 << 32 | k as u64) | 1);
                        let quiet = front_end(&mut db, &name, &src).len();
                        let copy = (0..20).map(|_| mutate(&src, &mut rng)).find(|c| {
                            c.len() >= src.len() && front_end(&mut db, &name, c).len() > quiet
                        });
                        let Some(copy) = copy else { continue };
                        let time = |db: &mut cpl_check::Session, text: &str| {
                            let start = Instant::now();
                            front_end(db, &name, text);
                            start.elapsed()
                        };
                        let (mut best_clean, mut best_broken) = (Duration::MAX, Duration::MAX);
                        for _ in 0..3 {
                            best_clean = best_clean.min(time(&mut db, &src));
                            best_broken = best_broken.min(time(&mut db, &copy));
                        }
                        clean += best_clean;
                        broken += best_broken;
                        copies += 1;
                    }
                    (clean, broken, copies)
                })
            })
            .collect();
        workers
            .into_iter()
            .map(|w| w.join().unwrap())
            .fold((Duration::ZERO, Duration::ZERO, 0), |a, b| {
                (a.0 + b.0, a.1 + b.1, a.2 + b.2)
            })
    });
    let ratio = broken.as_secs_f64() / clean.as_secs_f64();
    println!("{copies} broken copies: {broken:?} against {clean:?} clean, {ratio:.3}x");
    assert!(
        copies * 2 > paths.len(),
        "only {copies} of {} specs broke",
        paths.len()
    );
    assert!(
        ratio <= 1.10,
        "diagnostics add {:.0}% to the front end ({broken:?} against {clean:?})",
        (ratio - 1.0) * 100.0
    );
}
