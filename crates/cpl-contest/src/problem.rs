//! A problem on disk: `d.cp`, its limits, and the cases in `d.tests/`.

use std::path::{Path, PathBuf};
use std::time::Duration;

/// `d` and `d.cp` both name the problem `d.cp`, even beside a built `d`.
pub fn source_path(arg: &str) -> PathBuf {
    let path = PathBuf::from(arg);
    let source = path.with_extension("cp");
    if path.extension().is_some() || (path.is_file() && !source.is_file()) {
        path
    } else {
        source
    }
}

/// What a judge allows a run.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Limits {
    pub time: Duration,
    pub memory_kb: u64,
}

impl Default for Limits {
    fn default() -> Limits {
        Limits {
            time: Duration::from_millis(2000),
            memory_kb: 256 * 1024,
        }
    }
}

impl Limits {
    /// The line `cpl new` writes at the top of a problem.
    pub fn header(&self) -> String {
        format!(
            "// limits: {} ms, {} MB",
            self.time.as_millis(),
            self.memory_kb / 1024
        )
    }

    /// The limits of a problem: its `// limits: 2000 ms, 256 MB` line,
    /// else `[limits]` in the nearest `cp.toml`, else its judge's usual
    /// limits (2 s and 256 MB for `generic`).
    pub fn of(source: &Path, src: &str) -> Limits {
        let mut limits = match cpl_judge::resolve(source, src, None) {
            Ok(profile) => Limits {
                time: Duration::from_millis(profile.time_ms),
                memory_kb: profile.memory_mb * 1024,
            },
            Err(_) => Limits::default(),
        };
        let mut dir = source.parent();
        while let Some(parent) = dir {
            if let Ok(text) = std::fs::read_to_string(parent.join("cp.toml")) {
                limits.apply_toml(&text);
                break;
            }
            dir = parent.parent();
        }
        if let Some(line) = src
            .lines()
            .find_map(|line| line.trim().strip_prefix("// limits:"))
        {
            for part in line.split(',') {
                let mut words = part.split_whitespace();
                match (
                    words.next().and_then(|n| n.parse::<u64>().ok()),
                    words.next(),
                ) {
                    (Some(n), Some("ms")) => limits.time = Duration::from_millis(n),
                    (Some(n), Some("s")) => limits.time = Duration::from_secs(n),
                    (Some(n), Some("MB")) => limits.memory_kb = n * 1024,
                    _ => {}
                }
            }
        }
        limits
    }

    /// Reads `time_ms` and `memory_mb` from the `[limits]` table.
    fn apply_toml(&mut self, text: &str) {
        let mut in_limits = false;
        for line in text.lines() {
            let line = line.split('#').next().unwrap_or("").trim();
            if line.starts_with('[') {
                in_limits = line == "[limits]";
            } else if let (true, Some((key, value))) = (in_limits, line.split_once('=')) {
                match (key.trim(), value.trim().parse::<u64>()) {
                    ("time_ms", Ok(n)) => self.time = Duration::from_millis(n),
                    ("memory_mb", Ok(n)) => self.memory_kb = n * 1024,
                    _ => {}
                }
            }
        }
    }
}

/// One case in `<name>.tests/`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TestCase {
    /// The file name without `.in`: `1`, `stress-48213`.
    pub name: String,
    pub input: PathBuf,
    /// `None` for a case with no `.out`: it is run and its output shown.
    pub expected: Option<PathBuf>,
}

/// Where a problem's cases live: `d.tests/` for `d.cp`.
pub fn tests_dir(source: &Path) -> PathBuf {
    source.with_extension("tests")
}

/// The existing folder of cases for `source`: `<problem>.tests/` if it
/// exists, else a sibling `tests/` folder (as in `corpus/problems/<id>/tests/`),
/// else `<problem>.tests/`.
pub fn find_tests_dir(source: &Path) -> PathBuf {
    let own = tests_dir(source);
    if own.is_dir() {
        return own;
    }
    if let Some(parent) = source.parent() {
        let sibling = parent.join("tests");
        if sibling.is_dir() {
            return sibling;
        }
    }
    own
}

/// The problem's own checker, `checker.py <input> <output> <answer>` (exit 0
/// accepts): in the cases folder, or beside it when that is a sibling
/// `tests/` (as in `corpus/problems/<id>/`).
pub fn find_checker(source: &Path) -> Option<PathBuf> {
    let cases = find_tests_dir(source);
    let beside = match cases.file_name() {
        Some(name) if name == "tests" => cases.parent().map(|p| p.join("checker.py")),
        _ => None,
    };
    [Some(cases.join("checker.py")), beside]
        .into_iter()
        .flatten()
        .find(|path| path.is_file())
}

/// The cases in `dir`: every `*.in` paired with `.out` or `.ans` if present,
/// numbered samples first in order, then the rest by name.
pub fn cases_in(dir: &Path) -> Vec<TestCase> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut cases: Vec<TestCase> = entries
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.extension().is_some_and(|e| e == "in"))
        .map(|input| {
            let expected = [input.with_extension("out"), input.with_extension("ans")]
                .into_iter()
                .find(|p| p.is_file());
            TestCase {
                name: input
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
                expected,
                input,
            }
        })
        .collect();
    cases.sort_by(
        |a, b| match (a.name.parse::<u64>(), b.name.parse::<u64>()) {
            (Ok(x), Ok(y)) => x.cmp(&y).then_with(|| a.name.cmp(&b.name)),
            (Ok(_), Err(_)) => std::cmp::Ordering::Less,
            (Err(_), Ok(_)) => std::cmp::Ordering::Greater,
            (Err(_), Err(_)) => a.name.cmp(&b.name),
        },
    );
    cases
}

/// The cases of a problem: numbered samples first, in order, then the
/// rest by name.
pub fn cases(source: &Path) -> Vec<TestCase> {
    cases_in(&find_tests_dir(source))
}
