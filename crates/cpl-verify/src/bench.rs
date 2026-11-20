//! Toolchain latency benchmark (plan/40-tooling.md, plan/60-editor.md).
//!
//! Measures, for each benchmark program, the stages a contestant waits
//! on, against the budget for each:
//!
//! - check: `.cp` to C++ (the whole front end), 50 ms, and the size of
//!   the emitted file, 64 KB;
//! - debug run: the interpreter from source to output, 200 ms;
//! - release run: front end, `g++ -O2` with a precompiled header, and the
//!   run, 500 ms;
//! - LSP diagnostics after an edit and LSP hover, 100 ms each.
//!
//! A stage whose tool does not exist yet is reported as not measured. It
//! is never reported as passing.

use crate::backend::{Cxx, Interpreter, Transpiler};
use crate::json::Json;
use crate::lsp::Client;
use crate::process;
use std::fmt::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

pub const CHECK_BUDGET: Duration = Duration::from_millis(50);
pub const DEBUG_RUN_BUDGET: Duration = Duration::from_millis(200);
pub const RELEASE_RUN_BUDGET: Duration = Duration::from_millis(500);
pub const LSP_BUDGET: Duration = Duration::from_millis(100);
pub const EMITTED_BYTES_BUDGET: u64 = 64 * 1024;

const STEP_LIMIT: Duration = Duration::from_secs(60);

/// One benchmark program and the input it runs on.
#[derive(Debug, Clone)]
pub struct Program {
    pub name: String,
    pub source: PathBuf,
    pub stdin: Vec<u8>,
}

pub struct Tools<'a> {
    pub transpiler: &'a dyn Transpiler,
    /// `g++ -O2` with its precompiled header already built.
    pub release: &'a Cxx,
    pub interpreter: Option<&'a dyn Interpreter>,
    /// The command that starts the language server on stdio.
    pub lsp: Option<&'a [String]>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Outcome {
    /// One duration per run.
    Timed(Vec<Duration>),
    NotMeasured(String),
    Failed(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stage {
    pub program: String,
    pub name: &'static str,
    pub budget: Duration,
    pub outcome: Outcome,
}

impl Stage {
    pub fn median(&self) -> Option<Duration> {
        match &self.outcome {
            Outcome::Timed(runs) if !runs.is_empty() => {
                let mut sorted = runs.clone();
                sorted.sort();
                Some(sorted[sorted.len() / 2])
            }
            _ => None,
        }
    }

    /// Over budget, or broken. A stage that was not measured is neither.
    pub fn fails(&self) -> bool {
        match &self.outcome {
            Outcome::Timed(_) => self.median().is_some_and(|m| m > self.budget),
            Outcome::NotMeasured(_) => false,
            Outcome::Failed(_) => true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Report {
    pub stages: Vec<Stage>,
    /// (program, bytes of emitted C++).
    pub emitted_bytes: Vec<(String, u64)>,
}

impl Report {
    pub fn within_budget(&self) -> bool {
        !self.stages.iter().any(Stage::fails)
            && self
                .emitted_bytes
                .iter()
                .all(|(_, n)| *n <= EMITTED_BYTES_BUDGET)
    }

    pub fn render(&self) -> String {
        let mut out = String::new();
        let ms = |d: Duration| d.as_secs_f64() * 1000.0;
        for stage in &self.stages {
            write!(out, "{:12} {:16} ", stage.program, stage.name).unwrap();
            match &stage.outcome {
                Outcome::Timed(runs) => {
                    let median = stage.median().unwrap_or_default();
                    let verdict = if stage.fails() { "OVER" } else { "ok" };
                    writeln!(
                        out,
                        "median {:7.1} ms, worst {:7.1} ms, budget {:4.0} ms  {verdict}",
                        ms(median),
                        ms(runs.iter().copied().max().unwrap_or_default()),
                        ms(stage.budget)
                    )
                }
                Outcome::NotMeasured(why) => writeln!(out, "not measured: {why}"),
                Outcome::Failed(why) => writeln!(out, "FAILED: {why}"),
            }
            .unwrap();
        }
        for (program, bytes) in &self.emitted_bytes {
            let verdict = if *bytes > EMITTED_BYTES_BUDGET {
                "OVER"
            } else {
                "ok"
            };
            writeln!(
                out,
                "{program:12} {:16} {bytes} bytes, budget {EMITTED_BYTES_BUDGET} bytes  {verdict}",
                "emitted C++"
            )
            .unwrap();
        }
        out
    }
}

fn timed(runs: usize, mut step: impl FnMut() -> Result<Duration, String>) -> Outcome {
    let mut times = vec![];
    for _ in 0..runs {
        match step() {
            Ok(t) => times.push(t),
            Err(e) => return Outcome::Failed(e.lines().next().unwrap_or("").to_string()),
        }
    }
    Outcome::Timed(times)
}

/// Runs every stage `runs` times on every program. `work` is a scratch
/// directory.
pub fn run(tools: &Tools, programs: &[Program], runs: usize, work: &Path) -> Report {
    let mut report = Report {
        stages: vec![],
        emitted_bytes: vec![],
    };
    for program in programs {
        let mut stage = |name, budget, outcome| {
            report.stages.push(Stage {
                program: program.name.clone(),
                name,
                budget,
                outcome,
            })
        };
        let (cpp, exe) = (work.join("bench.cpp"), work.join("bench"));
        let check = || {
            let start = Instant::now();
            tools.transpiler.transpile(&program.source, &cpp)?;
            Ok(start.elapsed())
        };
        stage("check", CHECK_BUDGET, timed(runs, check));

        stage(
            "debug run",
            DEBUG_RUN_BUDGET,
            match tools.interpreter {
                None => Outcome::NotMeasured(
                    "no interpreter yet (cpl-interp cannot run programs)".into(),
                ),
                Some(interp) => timed(runs, || {
                    let out = interp
                        .run(&program.source, &program.stdin, STEP_LIMIT)
                        .map_err(|e| e.to_string())?;
                    if out.ok() {
                        Ok(out.elapsed)
                    } else {
                        Err(format!("exit {:?}: {}", out.exit, out.stderr))
                    }
                }),
            },
        );

        stage(
            "release run",
            RELEASE_RUN_BUDGET,
            timed(runs, || {
                let start = Instant::now();
                tools.transpiler.transpile(&program.source, &cpp)?;
                tools.release.compile(&cpp, &exe)?;
                let out = process::run(&mut Command::new(&exe), &program.stdin, STEP_LIMIT)
                    .map_err(|e| e.to_string())?;
                if out.ok() {
                    Ok(start.elapsed())
                } else {
                    Err(format!("exit {:?}: {}", out.exit, out.stderr))
                }
            }),
        );
        if let Ok(meta) = std::fs::metadata(&cpp) {
            report
                .emitted_bytes
                .push((program.name.clone(), meta.len()));
        }

        let (diagnostics, hover) = match tools.lsp {
            None => {
                let why = || {
                    Outcome::NotMeasured(
                        "no language server yet (cpl-lsp is an empty crate)".into(),
                    )
                };
                (why(), why())
            }
            Some(command) => match lsp_session(command, &program.source, runs) {
                Ok((d, h)) => (Outcome::Timed(d), Outcome::Timed(h)),
                Err(e) => (Outcome::Failed(e.clone()), Outcome::Failed(e)),
            },
        };
        let mut stage = |name, outcome| {
            report.stages.push(Stage {
                program: program.name.clone(),
                name,
                budget: LSP_BUDGET,
                outcome,
            })
        };
        stage("lsp diagnostics", diagnostics);
        stage("lsp hover", hover);
    }
    report
}

/// The position of the first name a `let`, `var` or `fn` declares, as
/// (line, character); the start of the file when there is none.
fn hover_position(text: &str) -> (usize, usize) {
    for (line, content) in text.lines().enumerate() {
        for keyword in ["let ", "var ", "fn "] {
            if let Some(rest) = content.trim_start().strip_prefix(keyword) {
                let column = content.len() - rest.len();
                return (line, column);
            }
        }
    }
    (0, 0)
}

/// Opens `source` in the server, then times `runs` edits (until the
/// diagnostics for the edit arrive) and `runs` hovers.
fn lsp_session(
    command: &[String],
    source: &Path,
    runs: usize,
) -> Result<(Vec<Duration>, Vec<Duration>), String> {
    let text = std::fs::read_to_string(source).map_err(|e| format!("{}: {e}", source.display()))?;
    let uri = format!("file://{}", source.display());
    let limit = Duration::from_secs(10);
    let mut client =
        Client::start(command).map_err(|e| format!("starting `{}`: {e}", command.join(" ")))?;
    let session = (|| {
        client.request(
            "initialize",
            Json::obj([
                ("processId", Json::Null),
                ("rootUri", Json::Null),
                ("capabilities", Json::obj([])),
            ]),
            limit,
        )?;
        client.notify("initialized", Json::obj([]))?;
        let document = |version: usize| {
            Json::obj([
                ("uri", Json::str(&uri)),
                ("version", Json::Num(version as f64)),
            ])
        };
        client.notify(
            "textDocument/didOpen",
            Json::obj([(
                "textDocument",
                Json::obj([
                    ("uri", Json::str(&uri)),
                    ("languageId", Json::str("cp")),
                    ("version", Json::Num(0.0)),
                    ("text", Json::str(&text)),
                ]),
            )]),
        )?;
        client.notification("textDocument/publishDiagnostics", limit)?;

        let mut diagnostics = vec![];
        for run in 1..=runs {
            // A keystroke: the whole document again with a trailing space
            // added or removed, so every edit changes the text.
            let edited = if run % 2 == 1 {
                format!("{text} ")
            } else {
                text.clone()
            };
            client.drain();
            let start = Instant::now();
            client.notify(
                "textDocument/didChange",
                Json::obj([
                    ("textDocument", document(run)),
                    (
                        "contentChanges",
                        Json::Arr(vec![Json::obj([("text", Json::str(&edited))])]),
                    ),
                ]),
            )?;
            client.notification("textDocument/publishDiagnostics", limit)?;
            diagnostics.push(start.elapsed());
        }

        let (line, character) = hover_position(&text);
        let mut hover = vec![];
        for _ in 0..runs {
            let start = Instant::now();
            client.request(
                "textDocument/hover",
                Json::obj([
                    ("textDocument", Json::obj([("uri", Json::str(&uri))])),
                    (
                        "position",
                        Json::obj([
                            ("line", Json::Num(line as f64)),
                            ("character", Json::Num(character as f64)),
                        ]),
                    ),
                ]),
                limit,
            )?;
            hover.push(start.elapsed());
        }
        Ok((diagnostics, hover))
    })();
    client.shutdown();
    session
}

/// The benchmark programs: every `<name>.in` in `inputs` is the input
/// for `<name>.cp` in `sources`.
pub fn programs(sources: &Path, inputs: &Path) -> Result<Vec<Program>, String> {
    let mut found = vec![];
    for entry in std::fs::read_dir(inputs).map_err(|e| format!("{}: {e}", inputs.display()))? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.extension().is_some_and(|e| e == "in") {
            let name = path.file_stem().unwrap().to_string_lossy().into_owned();
            let source = sources.join(format!("{name}.cp"));
            if !source.is_file() {
                return Err(format!(
                    "{} has no source at {}",
                    path.display(),
                    source.display()
                ));
            }
            let stdin = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            found.push(Program {
                name,
                source,
                stdin,
            });
        }
    }
    found.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(found)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stage(outcome: Outcome) -> Stage {
        Stage {
            program: "p".into(),
            name: "check",
            budget: Duration::from_millis(50),
            outcome,
        }
    }

    fn ms(values: &[u64]) -> Outcome {
        Outcome::Timed(values.iter().map(|&v| Duration::from_millis(v)).collect())
    }

    #[test]
    fn the_median_is_compared_with_the_budget() {
        // One slow run out of five is noise; three are a regression.
        assert!(!stage(ms(&[10, 400, 20, 30, 40])).fails());
        assert!(stage(ms(&[10, 60, 70, 80, 20])).fails());
        assert_eq!(
            stage(ms(&[30, 10, 20])).median(),
            Some(Duration::from_millis(20))
        );
    }

    #[test]
    fn a_missing_tool_is_not_a_pass_and_a_broken_one_is_a_failure() {
        let missing = stage(Outcome::NotMeasured("no interpreter yet".into()));
        assert!(!missing.fails());
        assert!(stage(Outcome::Failed("exit 1".into())).fails());
        let report = Report {
            stages: vec![missing],
            emitted_bytes: vec![("p".into(), 70_000)],
        };
        let text = report.render();
        assert!(text.contains("not measured: no interpreter yet"), "{text}");
        assert!(
            text.contains("70000 bytes, budget 65536 bytes  OVER"),
            "{text}"
        );
        assert!(!report.within_budget());
    }

    #[test]
    fn hover_lands_on_the_first_declared_name() {
        assert_eq!(hover_position("// c\n    let total = 0\n"), (1, 8));
        assert_eq!(hover_position("fn solve() -> Int:\n"), (0, 3));
        assert_eq!(hover_position("print(1)\n"), (0, 0));
    }
}
