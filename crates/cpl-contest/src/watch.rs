//! `cpl watch`: check, lint and test on every save.

use crate::problem::{cases_in, find_tests_dir, source_path};
use crate::test::{ready, render, results_json, run_cases, CaseResult};
use cpl_diag::Diagnostic;
use cpl_json::{obj, Value};
use cpl_stress::args::Args;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

pub const USAGE: &str = "\
usage: cpl watch <problem> [options]

On every save of the problem, its tests, or the `--fast` program: reports
errors, then lints, then runs the cases in <problem>.tests/.

options:
  --once             run one round and exit, with the round's exit code
  --fast PROG        run this instead of building the problem
  --checker PROG     judge with `PROG <input> <output> <answer>`; exit 0 accepts
  --interactor PROG  judge by talking to `PROG <input>`; exit 0 accepts
  --time-limit MS    per case (default: the problem's limit)
  --json             print each round as one line of JSON: `path`, `passed`,
                     the `diagnostics` (errors, or the lints when there are
                     none), `test` as `cpl test --json` prints it (null when
                     the cases didn't run), why they were `skipped`, and any
                     `error` that stopped the round
";

/// The state of the files a problem is made of: when each last changed.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Stamp(Vec<(PathBuf, Option<SystemTime>)>);

impl Stamp {
    /// Stamps the source, the `extra` programs, and everything in its
    /// tests folder. A file that appears or disappears changes the stamp.
    pub fn of(source: &Path, extra: &[PathBuf]) -> Stamp {
        let mut paths = vec![source.to_path_buf()];
        paths.extend(extra.iter().cloned());
        if let Ok(entries) = std::fs::read_dir(find_tests_dir(source)) {
            paths.extend(entries.filter_map(|entry| entry.ok().map(|e| e.path())));
        }
        paths.sort();
        Stamp(
            paths
                .into_iter()
                .map(|path| {
                    let modified = std::fs::metadata(&path).and_then(|m| m.modified()).ok();
                    (path, modified)
                })
                .collect(),
        )
    }
}

/// One problem being watched. The checker's database lives across
/// rounds, so a save only re-checks what changed.
pub struct Watch {
    source: PathBuf,
    db: cpl_check::Database,
    /// Print each round as one line of JSON instead of text.
    pub json: bool,
}

impl Watch {
    pub fn new(source: PathBuf) -> Watch {
        Watch {
            source,
            db: cpl_check::Database::new(),
            json: false,
        }
    }

    /// One round: errors, then lints, then tests, printed as text or, with
    /// `json`, as one line of JSON. Returns true when the source has no
    /// errors and every case passed.
    pub fn round(&mut self, args: &[String], out: &mut dyn Write) -> bool {
        let found = self.try_round(args);
        let passed = found.as_ref().is_ok_and(Round::passed);
        if self.json {
            let _ = writeln!(
                out,
                "{}",
                cpl_json::to_string(&self.json_of(&found, passed))
            );
            return passed;
        }
        match found {
            Ok(round) => self.render_text(&round, out),
            Err(message) => {
                let _ = writeln!(out, "  error: {message}");
            }
        }
        passed
    }

    fn try_round(&mut self, args: &[String]) -> Result<Round, String> {
        let name = self.source.shown();
        let text =
            std::fs::read_to_string(&self.source).map_err(|e| format!("can't read {name}: {e}"))?;
        self.db.set_file(&name, text.as_str());
        let errors = self.db.diagnostics(&name);
        let round = |diagnostics, errors, test| Round {
            text: text.clone(),
            diagnostics,
            errors,
            test,
        };
        if !errors.is_empty() {
            return Ok(round(errors, true, Tested::Stopped));
        }
        let (config, _) = cpl_lint::Config::discover(&self.source);
        let lints = cpl_lint::lint(&text, &config);

        let args = Args::parse("watch", args, VALUED, FLAGS)?;
        let dir = args
            .value("--tests")
            .map(PathBuf::from)
            .unwrap_or_else(|| find_tests_dir(&self.source));
        let cases = cases_in(&dir);
        if cases.is_empty() {
            let message = format!("no cases in {}", dir.shown());
            return Ok(round(lints, false, Tested::Skipped(message)));
        }
        let ready = match ready(&self.source, &text, &args) {
            Ok(ready) => ready,
            // A solution that can't be run yet is a state, not a failure
            // of the source.
            Err(message) => {
                return Ok(round(
                    lints,
                    false,
                    Tested::Skipped(format!("skipped: {message}")),
                ))
            }
        };
        let mut results = run_cases(&ready.solution, &ready.judge, &cases, ready.limits)
            .map_err(|e| format!("can't run {}: {e}", ready.solution.name))?;
        if args.value("--fast").is_none() {
            crate::test::explain_runtime_errors(&mut results, &cases, &self.source);
        }
        Ok(round(lints, false, Tested::Ran(results)))
    }

    fn render_text(&self, round: &Round, out: &mut dyn Write) {
        let name = self.source.shown();
        let shown = cpl_diag::render_all(&round.diagnostics, &name, &round.text);
        let count = round.diagnostics.len();
        if round.errors {
            let _ = write!(out, "{shown}");
            let _ = writeln!(out, "  check: {} to fix", plural(count, "error"));
            return;
        }
        let _ = writeln!(out, "  check: no errors");
        if count > 0 {
            let _ = write!(out, "{shown}");
            let _ = writeln!(out, "  lint: {}", plural(count, "warning"));
        }
        match &round.test {
            Tested::Stopped => {}
            Tested::Skipped(message) => {
                let _ = writeln!(out, "  test: {message}");
            }
            Tested::Ran(results) => {
                let _ = write!(out, "{}", render(results));
            }
        }
    }

    fn json_of(&self, found: &Result<Round, String>, passed: bool) -> Value {
        let text = |s: &str| Value::String(s.to_string());
        let (diagnostics, test, skipped, error) = match found {
            Ok(round) => {
                let lines = cpl_diag::LineIndex::new(&round.text);
                let diagnostics = round
                    .diagnostics
                    .iter()
                    .map(|d| cpl_diag::json::diagnostic(d, &lines))
                    .collect();
                let (test, skipped) = match &round.test {
                    Tested::Stopped => (Value::Null, Value::Null),
                    Tested::Skipped(message) => (Value::Null, text(message)),
                    Tested::Ran(results) => (results_json(results), Value::Null),
                };
                (diagnostics, test, skipped, Value::Null)
            }
            Err(message) => (Vec::new(), Value::Null, Value::Null, text(message)),
        };
        obj([
            ("path", text(&self.source.shown())),
            ("passed", Value::Bool(passed)),
            ("diagnostics", Value::Array(diagnostics)),
            ("test", test),
            ("skipped", skipped),
            ("error", error),
        ])
    }
}

/// What one round found.
struct Round {
    text: String,
    /// The errors when there are any, else the lints.
    diagnostics: Vec<Diagnostic>,
    errors: bool,
    test: Tested,
}

enum Tested {
    /// Errors stopped the round before the tests.
    Stopped,
    /// The cases could not run yet: why, as the text output says it.
    Skipped(String),
    Ran(Vec<CaseResult>),
}

impl Round {
    fn passed(&self) -> bool {
        !self.errors
            && match &self.test {
                Tested::Ran(results) => results.iter().all(CaseResult::passed),
                _ => true,
            }
    }
}

const FLAGS: &[&str] = &["--once", "--json"];

fn plural(count: usize, noun: &str) -> String {
    format!("{count} {noun}{}", if count == 1 { "" } else { "s" })
}

use crate::test::VALUED;
use cpl_diag::Shown;

/// Runs `cpl watch`. Prints as it goes, so it takes the real stdout.
pub fn run(args: &[String], stdout: &mut dyn Write, stderr: &mut dyn Write) -> u8 {
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        let _ = write!(stdout, "{USAGE}");
        return 0;
    }
    let parsed = Args::parse("watch", args, VALUED, FLAGS).and_then(|args| {
        args.file("watch")
            .map(source_path)
            .map(|source| (args, source))
    });
    let (parsed, source) = match parsed {
        Ok(parsed) => parsed,
        Err(message) => {
            let _ = write!(stderr, "cpl: {message}\n\n{USAGE}");
            return 2;
        }
    };
    if !source.is_file() {
        let _ = writeln!(stderr, "cpl: there is no file {}", source.shown());
        return 1;
    }
    let extra: Vec<PathBuf> = VALUED[..3]
        .iter()
        .filter_map(|flag| parsed.value(flag))
        .map(PathBuf::from)
        .collect();
    let mut watch = Watch::new(source.clone());
    watch.json = parsed.has("--json");
    let mut seen = Stamp::of(&source, &extra);
    let mut passed = watch.round(args, stdout);
    let _ = stdout.flush();
    if parsed.has("--once") {
        return !passed as u8;
    }
    loop {
        std::thread::sleep(Duration::from_millis(150));
        let now = Stamp::of(&source, &extra);
        if now == seen {
            continue;
        }
        seen = now;
        if !watch.json {
            let _ = writeln!(stdout, "\n--- {} changed", source.shown());
        }
        passed = watch.round(args, stdout);
        // A closed terminal ends the watch.
        if stdout.flush().is_err() {
            return !passed as u8;
        }
    }
}
