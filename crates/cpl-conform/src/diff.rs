use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::PoisonError;
use std::sync::{Condvar, Mutex};

use cpl_check::Session;

use crate::conformance::{self, Expect};
use crate::corpus;
use crate::runner::{self, CompileError};
use crate::scoreboard::{CaseOutcome, SuiteReport};
use crate::variants;
use cpl_stress::Verdict;

#[derive(Clone, Debug)]
enum DiffTarget {
    Conformance {
        id: String,
        path: PathBuf,
    },
    Corpus {
        id: String,
        spec_path: PathBuf,
        tests: Vec<PathBuf>,
        /// An interactive package's interactor: both engines are judged by it.
        interactor: Option<PathBuf>,
    },
    /// A corpus `port.cp` with side-effect probes (`variants`), on its first (sample) input.
    Variant {
        id: String,
        port_path: PathBuf,
        input: PathBuf,
        interactor: Option<PathBuf>,
    },
}

impl DiffTarget {
    fn id(&self) -> &str {
        match self {
            DiffTarget::Conformance { id, .. }
            | DiffTarget::Corpus { id, .. }
            | DiffTarget::Variant { id, .. } => id,
        }
    }
}

fn discover(root: &Path, filter: Option<&str>) -> Vec<DiffTarget> {
    let mut targets = Vec::new();

    for case in conformance::discover(root, None) {
        let id = format!("conformance/{}", case.id);
        if let Some(f) = filter {
            if !id.contains(f) {
                continue;
            }
        }
        let Ok(text) = std::fs::read_to_string(&case.path) else {
            continue;
        };
        if matches!(conformance::parse_header(&text), Ok(Expect::OkOutput(_))) {
            targets.push(DiffTarget::Conformance {
                id,
                path: case.path,
            });
        }
    }

    for prob in corpus::discover(root, None) {
        if prob.tests.is_empty() {
            continue;
        }
        let id = format!("corpus/{}", prob.id);
        if let Some(f) = filter {
            if !id.contains(f) {
                continue;
            }
        }
        targets.push(DiffTarget::Corpus {
            id,
            spec_path: prob.spec_path,
            tests: prob.tests.into_iter().map(|(inp, _)| inp).collect(),
            interactor: prob.interactor,
        });
    }

    for prob in corpus::discover_file(root, None, "port.cp") {
        let id = format!("variant/{}", prob.id);
        if filter.is_some_and(|f| !id.contains(f)) {
            continue;
        }
        // The package's first case, its sample: the fewest bytes is not the
        // least work (cf2124f2's 16-byte `5000 1` took the interpreter 33 s).
        if let Some((input, _)) = prob.tests.into_iter().min() {
            targets.push(DiffTarget::Variant {
                id,
                port_path: prob.spec_path,
                input,
                interactor: prob.interactor,
            });
        }
    }

    let mut mine: Vec<DiffTarget> = targets
        .into_iter()
        .enumerate()
        .filter(|(i, _)| runner::in_shard(*i))
        .map(|(_, t)| t)
        .collect();
    // Problems run real judge inputs and take up to a minute; queued after
    // the thousand small conformance cases, the last one ran alone at the
    // end of the shard. Started first, the small cases fill in around them.
    mine.sort_by_key(|t| matches!(t, DiffTarget::Conformance { .. }));
    mine
}

fn fnv1a_hex(data: &[u8]) -> String {
    let mut h1: u64 = 0xcbf29ce484222325;
    let mut h2: u64 = 0x84222325cbf29ce4;
    for &b in data {
        h1 ^= u64::from(b);
        h1 = h1.wrapping_mul(0x100000001b3);
        h2 = h2.wrapping_mul(0x100000001b3);
        h2 ^= u64::from(b);
    }
    format!("{h1:016x}{h2:016x}")
}

pub(crate) enum CppCompileError {
    Cxx(String),
    Infra(String),
}

struct GxxGate {
    active: Mutex<usize>,
    cv: Condvar,
    max: usize,
}

impl GxxGate {
    const fn new(max: usize) -> Self {
        Self {
            active: Mutex::new(0),
            cv: Condvar::new(),
            max,
        }
    }

    fn acquire(&self) -> GxxPermit<'_> {
        let mut active = self.active.lock().unwrap_or_else(PoisonError::into_inner);
        while *active >= self.max {
            active = self.cv.wait(active).unwrap_or_else(PoisonError::into_inner);
        }
        *active += 1;
        GxxPermit(self)
    }
}

struct GxxPermit<'a>(&'a GxxGate);

impl Drop for GxxPermit<'_> {
    fn drop(&mut self) {
        let mut active = self.0.active.lock().unwrap_or_else(PoisonError::into_inner);
        *active = active.saturating_sub(1);
        self.0.cv.notify_one();
    }
}

static GXX_GATE: GxxGate = GxxGate::new(6);

fn is_transient_gxx_failure(code: Option<i32>, stderr: &str) -> bool {
    if code.is_none() {
        return true;
    }
    let lower = stderr.to_ascii_lowercase();
    lower.contains("cannot allocate memory")
        || lower.contains("resource temporarily unavailable")
        || lower.contains("killed")
        || lower.contains("fork:")
        || lower.contains("virtual memory exhausted")
}

pub(crate) fn compile_cpp(cpp: &str, cache_dir: &Path) -> Result<PathBuf, CppCompileError> {
    // `CPL_CONFORM_CXXFLAGS` replaces `-O2`, e.g. `-O1 -g -fsanitize=address,undefined`
    // to turn undefined behaviour in emitted C++ into a failing case.
    let flags = cpl_env::CONFORM_CXXFLAGS
        .text()
        .unwrap_or_else(|| "-O2".into());
    compile_cpp_with(cpp, cache_dir, &flags)
}

/// `compile_cpp` with `flags` in place of `-O2`.
pub(crate) fn compile_cpp_with(
    cpp: &str,
    cache_dir: &Path,
    flags: &str,
) -> Result<PathBuf, CppCompileError> {
    std::fs::create_dir_all(cache_dir)
        .map_err(|e| CppCompileError::Infra(format!("cannot create cache dir: {e}")))?;
    // `-Werror=return-type`: a fn that falls off its end is undefined
    // behaviour g++ only warns about; emitted C++ never does it.
    let flags = format!("{flags} -Werror=return-type");
    // `CPL_CXX_SYSTEM_KEY` fingerprints the compiler and system headers when the
    // cache outlives them (shared across checkouts and CI runs).
    let system = cpl_env::CXX_SYSTEM_KEY.text().unwrap_or_default();
    let key = fnv1a_hex(format!("{system}\0{flags}\0{cpp}").as_bytes());
    let bin = cache_dir.join(&key);
    if bin.is_file() {
        // Used: CI's cache trim (scripts/ci.py cxx-trim) keeps it.
        let _ =
            std::fs::File::open(&bin).and_then(|f| f.set_modified(std::time::SystemTime::now()));
        return Ok(bin);
    }
    // A worker building the same binary holds its lock; the others wait on
    // it and then find the binary built.
    let _lock = std::fs::File::create(cache_dir.join(format!("{key}.lock")))
        .and_then(|f| f.lock().map(|()| f))
        .map_err(|e| CppCompileError::Infra(format!("cannot lock the build of {key}: {e}")))?;
    if bin.is_file() {
        return Ok(bin);
    }
    let tid = std::thread::current().id();
    let unique = format!("{key}_{}_{tid:?}", std::process::id())
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();
    let src = cache_dir.join(format!("{unique}.cpp"));
    let tmp_bin = cache_dir.join(format!("{unique}.bin"));
    std::fs::write(&src, cpp)
        .map_err(|e| CppCompileError::Infra(format!("cannot write cpp: {e}")))?;

    let mut last_infra: Option<String> = None;
    for _ in 0..3 {
        let _permit = GXX_GATE.acquire();
        let out = match Command::new("g++")
            .args(flags.split_whitespace())
            .args(["-std=c++17", "-o"])
            .arg(&tmp_bin)
            .arg(&src)
            .output()
        {
            Ok(o) => o,
            Err(e) => {
                last_infra = Some(format!("cannot run g++: {e}"));
                continue;
            }
        };
        if out.status.success() {
            let _ = std::fs::remove_file(&src);
            let _ = std::fs::rename(&tmp_bin, &bin);
            return if bin.is_file() { Ok(bin) } else { Ok(tmp_bin) };
        }
        let _ = std::fs::remove_file(&tmp_bin);
        let stderr = String::from_utf8_lossy(&out.stderr);
        if is_transient_gxx_failure(out.status.code(), &stderr) {
            last_infra = Some(format!(
                "g++ transient failure ({:?}): {}",
                out.status.code(),
                runner::first_line_trimmed(&stderr, 160)
            ));
            continue;
        }
        let _ = std::fs::remove_file(&src);
        return Err(CppCompileError::Cxx(runner::first_line_trimmed(
            &stderr, 160,
        )));
    }
    let _ = std::fs::remove_file(&src);
    let _ = std::fs::remove_file(&tmp_bin);
    Err(CppCompileError::Infra(
        last_infra.unwrap_or_else(|| "g++ failed after retries".into()),
    ))
}

pub(crate) fn run_binary(bin: &Path, input: &[u8]) -> Result<Vec<u8>, String> {
    let out = runner::run_piped(Command::new(bin), input, runner::case_timeout())
        .map_err(|e| format!("{}: {e}", bin.display()))?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(format!(
            "exit {:?}: {}",
            out.status.code(),
            runner::first_line_trimmed(&stderr, 120)
        ));
    }
    Ok(out.stdout)
}

fn run_one(db: &mut Session, cache_dir: &Path, target: &DiffTarget) -> (String, CaseOutcome) {
    let (id, path, inputs, token_compare) = match target {
        DiffTarget::Conformance { id, path } => (
            id.clone(),
            path.clone(),
            vec![(
                String::new(),
                std::fs::read_to_string(path)
                    .map(|t| conformance::header_input(&t))
                    .unwrap_or_default(),
            )],
            false,
        ),
        DiffTarget::Corpus {
            id,
            spec_path,
            tests,
            ..
        } => {
            let mut inps = Vec::with_capacity(tests.len());
            for t in tests {
                let name = t
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                match std::fs::read(t) {
                    Ok(bytes) => inps.push((name, bytes)),
                    Err(e) => {
                        return (
                            id.clone(),
                            CaseOutcome {
                                status: "error".into(),
                                detail: format!("cannot read {name}: {e}"),
                            },
                        );
                    }
                }
            }
            (id.clone(), spec_path.clone(), inps, true)
        }
        DiffTarget::Variant {
            id,
            port_path,
            input,
            ..
        } => match std::fs::read(input) {
            Ok(bytes) => (
                id.clone(),
                port_path.clone(),
                vec![(
                    input
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default(),
                    bytes,
                )],
                true,
            ),
            Err(e) => {
                return (
                    id.clone(),
                    CaseOutcome {
                        status: "error".into(),
                        detail: format!("cannot read the input: {e}"),
                    },
                );
            }
        },
    };

    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => {
            return (
                id,
                CaseOutcome {
                    status: "error".into(),
                    detail: format!("cannot read: {e}"),
                },
            );
        }
    };
    let path_str = path.to_string_lossy();
    let interactor = match target {
        DiffTarget::Corpus { interactor, .. } | DiffTarget::Variant { interactor, .. } => {
            interactor.as_deref()
        }
        DiffTarget::Conformance { .. } => None,
    };
    // What `cpl run` runs: the port, or for a variant the written variant.
    let mut run_path = path.clone();
    let text = if matches!(target, DiffTarget::Variant { .. }) {
        let skip = |detail: &str| CaseOutcome {
            status: "skip".into(),
            detail: detail.into(),
        };
        if runner::compile_program(db, &path_str, &text).is_err() {
            return (id, skip("the port does not compile yet"));
        }
        let input = inputs.first().map_or(&[][..], |(_, b)| b.as_slice());
        // An interactive port runs only against its interactor, so its probes are not pruned.
        let built = if interactor.is_some() {
            variants::build(db, &path_str, &text)
        } else {
            variants::build_running(db, &path_str, &text, input)
        };
        // With no probe that checks, the variant is the port itself: the corpus target
        // diffs spec.cp, so this is the only diff (and g++ build) a port gets.
        match built {
            None => text,
            Some((variant, _)) => {
                // Kept for reproducing a mismatch: `cpl run` it next to the port.
                let dir = cache_dir.join("variants");
                let written = dir.join(format!("{}.cp", id.replace('/', "_")));
                if std::fs::create_dir_all(&dir)
                    .and_then(|_| std::fs::write(&written, &variant))
                    .is_ok()
                {
                    run_path = written;
                } else if interactor.is_some() {
                    return (
                        id,
                        CaseOutcome {
                            status: "error".into(),
                            detail: format!("cannot write {}", written.display()),
                        },
                    );
                }
                variant
            }
        }
    } else {
        text
    };
    let checks = release_checks(&text);
    let compiled = match runner::compile_program(db, &path_str, &text) {
        Ok(c) => c,
        Err(CompileError::Diagnostics(_)) => {
            // Guard against cross-case Database state contamination under parallel reuse:
            // retry once on a fresh Database before classifying as skip.
            let mut fresh = Session::new();
            match runner::compile_program(&mut fresh, &path_str, &text) {
                Ok(c) => c,
                Err(CompileError::Diagnostics(diag)) => {
                    return (
                        id,
                        CaseOutcome {
                            status: "skip".into(),
                            detail: format!(
                                "does not compile yet: {}",
                                runner::first_line_trimmed(&diag, 140)
                            ),
                        },
                    );
                }
            }
        }
    };

    let mut interp_outputs = Vec::with_capacity(inputs.len());
    // An `@cpp` body runs only in C++: a case's written output stands in
    // for the interpreter's run (decision 449).
    let written = match conformance::parse_header(&text) {
        Ok(Expect::OkOutput(want)) if !compiled.program.cpp_bodies.is_empty() => Some(want),
        _ => None,
    };
    for (_, inp) in inputs.iter().filter(|_| interactor.is_none()) {
        if let Some(want) = &written {
            interp_outputs.push(want.clone().into_bytes());
            continue;
        }
        let limit = runner::case_timeout();
        let at = checks.unwrap_or(cpl_codegen::Checks::LOCAL);
        let run = match runner::run_interp_at(&compiled.program, &path_str, &text, inp, limit, at) {
            Ok(run) => run,
            Err(detail) => {
                return (
                    id,
                    CaseOutcome {
                        status: "error".into(),
                        detail,
                    },
                )
            }
        };
        if run.trap.is_some() {
            return (
                id,
                CaseOutcome {
                    status: "skip".into(),
                    detail: "interpreter trapped".into(),
                },
            );
        }
        interp_outputs.push(run.stdout);
    }

    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("main.cp");
    let target = cpl_codegen::Target {
        checks: checks.unwrap_or_default(),
        ..cpl_codegen::Target::default()
    };
    let cpp = cpl_codegen::emit_for(&compiled.program, name, Some(&text), &target);
    let bin = match compile_cpp(&cpp, cache_dir) {
        Ok(b) => b,
        Err(CppCompileError::Cxx(err)) => {
            return (
                id,
                CaseOutcome {
                    status: "cxx_err".into(),
                    detail: err,
                },
            );
        }
        Err(CppCompileError::Infra(err)) => {
            return (
                id,
                CaseOutcome {
                    status: "infra".into(),
                    detail: err,
                },
            );
        }
    };

    if let Some(interactor) = interactor {
        let cpl = match conformance::cli_binary() {
            Ok(cpl) => cpl.display().to_string(),
            Err(detail) => {
                return (
                    id,
                    CaseOutcome {
                        status: "error".into(),
                        detail,
                    },
                )
            }
        };
        let (spec, bin) = (run_path.display().to_string(), bin.display().to_string());
        let interp = cpl_stress::Program::command(&spec, &[&cpl, "run", &spec]);
        let cpp = cpl_stress::Program::command(&bin, &[&bin]);
        return (id, interactive_outcome(interactor, &interp, &cpp, &inputs));
    }
    for ((inp_name, inp), interp_out) in inputs.iter().zip(interp_outputs.iter()) {
        let cpp_out = match run_binary(&bin, inp) {
            Ok(o) => o,
            Err(err) => {
                return (
                    id,
                    CaseOutcome {
                        status: "cpp_trap".into(),
                        detail: if inp_name.is_empty() {
                            err
                        } else {
                            format!("{inp_name}: {err}")
                        },
                    },
                );
            }
        };
        let interp_s = String::from_utf8_lossy(interp_out);
        let cpp_s = String::from_utf8_lossy(&cpp_out);
        let equal = if token_compare {
            interp_s.split_whitespace().eq(cpp_s.split_whitespace())
        } else {
            interp_s.trim_end_matches('\n') == cpp_s.trim_end_matches('\n')
        };
        if !equal {
            return (
                id,
                CaseOutcome {
                    status: "mismatch".into(),
                    detail: format!(
                        "{}interp={:?} cpp={:?}",
                        if inp_name.is_empty() {
                            String::new()
                        } else {
                            format!("{inp_name}: ")
                        },
                        runner::truncate_chars(interp_s.trim(), 80),
                        runner::truncate_chars(cpp_s.trim(), 80),
                    ),
                },
            );
        }
    }

    (
        id,
        CaseOutcome {
            status: "pass".into(),
            detail: String::new(),
        },
    )
}

/// Whether two runs against an interactor agree: the same verdict and the same
/// transcript, except that after the interactor stops a run, how much more it
/// printed depends on timing, so only the shorter transcript must match.
fn agree(a: &cpl_stress::Judged, b: &cpl_stress::Judged) -> bool {
    if std::mem::discriminant(&a.verdict) != std::mem::discriminant(&b.verdict) {
        return false;
    }
    let (x, y) = (
        a.outcome.stdout.split_whitespace(),
        b.outcome.stdout.split_whitespace(),
    );
    if a.verdict == Verdict::Accepted {
        x.eq(y)
    } else {
        x.zip(y).all(|(p, q)| p == q)
    }
}

/// An interactive package, judged by its interactor under `cpl run` and as
/// the built C++: the verdicts and what each printed must agree on every case.
fn interactive_outcome(
    interactor: &Path,
    interp: &cpl_stress::Program,
    cpp: &cpl_stress::Program,
    inputs: &[(String, Vec<u8>)],
) -> CaseOutcome {
    let outcome = |status: &str, detail: String| CaseOutcome {
        status: status.into(),
        detail,
    };
    for (name, inp) in inputs {
        let inp = String::from_utf8_lossy(inp);
        match (
            corpus::interacted(interactor, interp, &inp),
            corpus::interacted(interactor, cpp, &inp),
        ) {
            (Ok(a), Ok(b)) if agree(&a, &b) => {}
            (Ok(a), Ok(b)) => {
                let (al, bl) = (a.outcome.stdout.lines(), b.outcome.stdout.lines());
                let at = al
                    .clone()
                    .zip(bl.clone())
                    .take_while(|(x, y)| x == y)
                    .count();
                let line = |mut l: std::str::Lines, v: &Verdict| {
                    format!(
                        "{:?} {v:?}",
                        runner::truncate_chars(l.nth(at).unwrap_or(""), 50)
                    )
                };
                let detail = format!(
                    "{name}: line {}: interp {}, cpp {}",
                    at + 1,
                    line(al, &a.verdict),
                    line(bl, &b.verdict),
                );
                return outcome("mismatch", runner::first_line_trimmed(&detail, 240));
            }
            (Err(e), _) | (_, Err(e)) => return outcome("error", format!("{name}: {e}")),
        }
    }
    outcome("pass", String::new())
}

pub fn run(root: &Path, filter: Option<&str>) -> SuiteReport {
    let targets = discover(root, filter);
    // The per-user g++ cache (scripts/check.sh, kept across CI runs) when there is one,
    // so a fresh checkout or CI job doesn't recompile every case.
    let cache_dir = match cpl_env::CXX_CACHE_DIR.path() {
        Some(dir) => dir.join("conform-diff"),
        None => std::env::var_os("CARGO_TARGET_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| root.join("target"))
            .join("conform-cxx"),
    };

    let one = |db: &mut Session, target: &DiffTarget| {
        runner::running(target.id());
        let (id, outcome) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            run_one(db, &cache_dir, target)
        }))
        .unwrap_or_else(|_| {
            let id = target.id().to_string();
            (
                id,
                CaseOutcome {
                    status: "panic".into(),
                    detail: "codegen/interp panicked".into(),
                },
            )
        });
        (id, outcome)
    };
    let mut outcomes = runner::parallel_with_db(&targets, one);
    rerun_timed_out_alone(&mut outcomes, &targets, DiffTarget::id, |again| {
        runner::with_db_on(1, again, |db, t| one(db, t))
    });
    SuiteReport::from_outcomes(outcomes)
}

/// Runs each case that timed out again, one at a time with the machine to
/// itself, and keeps that answer. Next to every other case on a loaded
/// runner, a case well inside its limit run alone can still pass it; a case
/// that is really too slow or never stops times out again.
fn rerun_timed_out_alone<'a, T>(
    outcomes: &mut BTreeMap<String, CaseOutcome>,
    items: &'a [T],
    id: impl Fn(&T) -> &str,
    rerun: impl FnOnce(&[&'a T]) -> BTreeMap<String, CaseOutcome>,
) {
    let timed_out: Vec<&T> = items
        .iter()
        .filter(|item| {
            outcomes
                .get(id(item))
                .is_some_and(|o| o.status == "error" && o.detail.contains("timed out after"))
        })
        .collect();
    if !timed_out.is_empty() {
        outcomes.extend(rerun(&timed_out));
    }
}

#[cfg(test)]
mod rerun_tests {
    use super::*;

    fn outcome(status: &str, detail: &str) -> CaseOutcome {
        CaseOutcome {
            status: status.into(),
            detail: detail.into(),
        }
    }

    #[test]
    fn only_timed_out_cases_run_again_and_keep_the_second_answer() {
        let mut outcomes = BTreeMap::from([
            (
                "slow".to_string(),
                outcome("error", "the interpreter timed out after 20s"),
            ),
            (
                "hung".to_string(),
                outcome("error", "the interpreter timed out after 20s"),
            ),
            ("wrong".to_string(), outcome("fail", "outputs differ")),
            ("ok".to_string(), outcome("pass", "")),
        ]);
        let items = ["slow", "hung", "wrong", "ok"];
        let mut reran = Vec::new();
        rerun_timed_out_alone(
            &mut outcomes,
            &items,
            |s| s,
            |again| {
                reran = again.iter().map(|s| **s).collect();
                again
                    .iter()
                    .map(|s| match **s {
                        "slow" => (s.to_string(), outcome("pass", "")),
                        _ => (
                            s.to_string(),
                            outcome("error", "the interpreter timed out after 20s"),
                        ),
                    })
                    .collect()
            },
        );
        assert_eq!(reran, ["slow", "hung"]);
        assert_eq!(outcomes["slow"].status, "pass");
        assert_eq!(outcomes["hung"].status, "error");
        assert_eq!(outcomes["wrong"].status, "fail");
    }
}

/// The checks a `// tool: run --release [--checks LEVEL]` case runs at on
/// both sides: release against release (R09.2.2), the judge build's level
/// (`none` unless it says) in the interpreter as in the C++. `None` for any
/// other case: the interpreter checks everything, as `cpl run` does.
fn release_checks(text: &str) -> Option<cpl_codegen::Checks> {
    let words: Vec<&str> = conformance::header_values(text, "tool:")
        .first()
        .map(|t| t.split_whitespace().collect())
        .unwrap_or_default();
    if words.first() != Some(&"run") || !words.contains(&"--release") {
        return None;
    }
    match words.iter().position(|w| *w == "--checks") {
        Some(i) => words.get(i + 1).and_then(|l| cpl_codegen::Checks::parse(l)),
        None => Some(cpl_codegen::Checks::None),
    }
}

#[cfg(test)]
mod compile_tests {
    use super::*;

    #[test]
    fn a_fn_that_falls_off_its_end_fails_to_compile() {
        let cache = std::env::temp_dir().join(format!("cpl-return-type-{}", std::process::id()));
        let missing = "long long f(long long x) { x += 1; }\nint main() { return (int)f(1); }\n";
        assert!(matches!(
            compile_cpp(missing, &cache),
            Err(CppCompileError::Cxx(_))
        ));
        let returns =
            "long long f(long long x) { return x + 1; }\nint main() { return (int)f(-1); }\n";
        assert!(compile_cpp(returns, &cache).is_ok());
        let _ = std::fs::remove_dir_all(cache);
    }
}

#[cfg(test)]
mod release_checks_tests {
    use super::release_checks;
    use cpl_codegen::Checks;

    #[test]
    fn a_release_case_runs_its_level_on_both_sides() {
        assert_eq!(
            release_checks("// tool: run --release --checks none\n"),
            Some(Checks::None)
        );
        assert_eq!(
            release_checks("// tool: run --release --checks traps\n"),
            Some(Checks::Traps)
        );
        assert_eq!(
            release_checks("// tool: run --release\n"),
            Some(Checks::None)
        );
        assert_eq!(release_checks("// expect: ok\n"), None);
    }
}

#[cfg(test)]
mod interactive_tests {
    use super::interactive_outcome;
    use cpl_stress::Program;
    use std::path::Path;

    #[test]
    fn an_interactive_package_is_judged_by_its_interactor_on_both_sides() {
        let dir = std::env::temp_dir().join(format!("cpl-diff-interactive-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let lib = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus/lib");
        let interactor = dir.join("interactor.py");
        std::fs::write(
            &interactor,
            format!(
                "import sys\nsys.path.insert(0, {:?})\nfrom interactor_lib import run_interactor\n\n\
                 def drive(case_text, io):\n    n = int(case_text)\n    io.send(str(n))\n    \
                 got = io.recv_line().strip()\n    \
                 if got != str(2 * n):\n        raise RuntimeError(f\"answer {{got}}, want {{2 * n}}\")\n\n\
                 run_interactor(drive)\n",
                lib.display().to_string()
            ),
        )
        .unwrap();
        let solve = |body: &str| Program::command(body, &["python3", "-c", body]);
        let right = solve("print(2 * int(input()))");
        let wrong = solve("print(int(input()))");
        let inputs = vec![("01.in".to_string(), b"3\n".to_vec())];
        assert_eq!(
            interactive_outcome(&interactor, &right, &right, &inputs).status,
            "pass"
        );
        let split = interactive_outcome(&interactor, &right, &wrong, &inputs);
        assert_eq!(split.status, "mismatch", "{}", split.detail);
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[cfg(test)]
mod gate_tests {
    use super::*;

    #[test]
    fn a_permit_dropped_by_a_panic_frees_its_place() {
        let gate = GxxGate::new(1);
        let panicked = std::thread::scope(|scope| {
            scope
                .spawn(|| {
                    let _permit = gate.acquire();
                    let _active = gate.active.lock();
                    panic!("a g++ run panicked holding the gate");
                })
                .join()
        });
        assert!(panicked.is_err());
        assert!(gate.active.is_poisoned());
        drop(gate.acquire());
        assert_eq!(
            *gate.active.lock().unwrap_or_else(PoisonError::into_inner),
            0
        );
    }
}
