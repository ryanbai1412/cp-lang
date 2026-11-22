//! `cpl watch`: check, lint and test on every save.

use cpl_stress::args::Args;
use crate::problem::{cases_in, find_tests_dir, source_path};
use crate::test::{ready, render, run_cases};
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
}

impl Watch {
    pub fn new(source: PathBuf) -> Watch {
        Watch {
            source,
            db: cpl_check::Database::new(),
        }
    }

    /// One round: errors, then lints, then tests. Returns true when the
    /// source has no errors and every case passed.
    pub fn round(&mut self, args: &[String], out: &mut dyn Write) -> bool {
        self.try_round(args, out).unwrap_or_else(|message| {
            let _ = writeln!(out, "  error: {message}");
            false
        })
    }

    fn try_round(&mut self, args: &[String], out: &mut dyn Write) -> Result<bool, String> {
        let name = self.source.display().to_string();
        let text =
            std::fs::read_to_string(&self.source).map_err(|e| format!("can't read {name}: {e}"))?;
        self.db.set_file(&name, text.as_str());
        let errors = self.db.diagnostics(&name);
        if !errors.is_empty() {
            let _ = write!(out, "{}", cpl_diag::render_all(&errors, &name, &text));
            let _ = writeln!(out, "  check: {} to fix", plural(errors.len(), "error"));
            return Ok(false);
        }
        let _ = writeln!(out, "  check: no errors");

        let (config, _) = cpl_lint::Config::discover(&self.source);
        let lints = cpl_lint::lint(&text, &config);
        if !lints.is_empty() {
            let _ = write!(out, "{}", cpl_diag::render_all(&lints, &name, &text));
            let _ = writeln!(out, "  lint: {}", plural(lints.len(), "warning"));
        }

        let args = Args::parse(args, VALUED, &["--once"])?;
        let dir = args
            .value("--tests")
            .map(PathBuf::from)
            .unwrap_or_else(|| find_tests_dir(&self.source));
        let cases = cases_in(&dir);
        if cases.is_empty() {
            let _ = writeln!(out, "  test: no cases in {}", dir.display());
            return Ok(true);
        }
        let ready = match ready(&self.source, &text, &args) {
            Ok(ready) => ready,
            // A solution that can't be run yet is a state, not a failure
            // of the source.
            Err(message) => {
                let _ = writeln!(out, "  test: skipped: {message}");
                return Ok(true);
            }
        };
        let mut results = run_cases(&ready.solution, &ready.judge, &cases, ready.limits)
            .map_err(|e| format!("can't run {}: {e}", ready.solution.name))?;
        if args.value("--fast").is_none() {
            crate::test::explain_runtime_errors(&mut results, &cases, &self.source);
        }
        let _ = write!(out, "{}", render(&results));
        Ok(results.iter().all(|result| result.passed()))
    }
}

fn plural(count: usize, noun: &str) -> String {
    format!("{count} {noun}{}", if count == 1 { "" } else { "s" })
}

use crate::test::VALUED;

/// Runs `cpl watch`. Prints as it goes, so it takes the real stdout.
pub fn run(args: &[String], stdout: &mut dyn Write, stderr: &mut dyn Write) -> u8 {
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        let _ = write!(stdout, "{USAGE}");
        return 0;
    }
    let parsed = Args::parse(args, VALUED, &["--once"]).and_then(|args| {
        args.file("watch")
            .map(source_path)
            .map(|source| (args, source))
    });
    let (parsed, source) = match parsed {
        Ok(parsed) => parsed,
        Err(message) => {
            let _ = write!(stderr, "error: {message}\n\n{USAGE}");
            return 2;
        }
    };
    if !source.is_file() {
        let _ = writeln!(stderr, "error: there is no file {}", source.display());
        return 1;
    }
    let extra: Vec<PathBuf> = VALUED[..3]
        .iter()
        .filter_map(|flag| parsed.value(flag))
        .map(PathBuf::from)
        .collect();
    let mut watch = Watch::new(source.clone());
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
        let _ = writeln!(stdout, "\n--- {} changed", source.display());
        passed = watch.round(args, stdout);
        // A closed terminal ends the watch.
        if stdout.flush().is_err() {
            return !passed as u8;
        }
    }
}
