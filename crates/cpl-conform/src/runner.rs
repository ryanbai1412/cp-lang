use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Instant;

use cpl_check::Database;
use cpl_diag::{Diagnostic, LineIndex, Severity};

/// Runs `cpl check` semantics, returning rendered
/// diagnostics and whether any error diagnostic occurred.
pub fn check_spec(db: &mut Database, path: &str, text: &str) -> (String, bool) {
    db.set_file(path, text);
    let analysis = db.analyze(path).expect("the file was just set");
    db.remove_file(path);
    let diagnostics: Vec<Diagnostic> = cpl_ir::reported(&analysis);
    let failed = diagnostics.iter().any(|d| d.severity == Severity::Error);
    let rendered = if diagnostics.is_empty() {
        String::new()
    } else {
        cpl_diag::render_all(&diagnostics, path, text)
    };
    (rendered, failed)
}

/// What `cpl lint` reports for `text` at `path`, rendered: an `L` code in
/// a case header names a lint, not a check error.
pub fn lint_spec(path: &str, text: &str) -> String {
    let (config, _) = cpl_lint::Config::discover(Path::new(path));
    let lints = cpl_lint::lint(text, &config);
    if lints.is_empty() {
        String::new()
    } else {
        cpl_diag::render_all(&lints, path, text)
    }
}

pub enum CompileError {
    Diagnostics(String),
}

pub struct CompiledProgram {
    pub program: cpl_ir::Program,
    pub warnings: String,
}

/// Checks and lowers `text` at `path` into `cpl_ir::Program`, matching `cpl run` / `cpl build`.
pub fn compile_program(
    db: &mut Database,
    path: &str,
    text: &str,
) -> Result<CompiledProgram, CompileError> {
    db.set_file(path, text);
    let analysis = db.analyze(path).expect("the file was just set");
    db.remove_file(path);
    if !analysis.diagnostics.is_empty()
        && analysis
            .diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error)
    {
        return Err(CompileError::Diagnostics(cpl_diag::render_all(
            &analysis.diagnostics,
            path,
            text,
        )));
    }
    match cpl_ir::lower(&analysis) {
        Ok(program) => {
            let warnings = if program.warnings.is_empty() {
                String::new()
            } else {
                cpl_diag::render_all(&program.warnings, path, text)
            };
            Ok(CompiledProgram { program, warnings })
        }
        Err(diags) => Err(CompileError::Diagnostics(cpl_diag::render_all(
            &diags, path, text,
        ))),
    }
}

pub struct RunOutput {
    pub stdout: Vec<u8>,
    pub trap: Option<cpl_interp::Trap>,
    pub trap_rendered: String,
}

/// Runs a lowered program in the debug interpreter on `input`.
pub fn run_interp(program: &cpl_ir::Program, path: &str, text: &str, input: &[u8]) -> RunOutput {
    let mut stdout = Vec::new();
    let observed =
        cpl_interp::run_observed(program, cpl_interp::Io::fixed(input), &mut stdout, None);
    let trap_rendered = match &observed.trap {
        Some(trap) => render_trap(path, text, trap),
        None => String::new(),
    };
    RunOutput {
        stdout,
        trap: observed.trap,
        trap_rendered,
    }
}

/// Seconds a case may run, from `CPL_CONFORM_CASE_TIMEOUT` (default 20).
pub fn case_timeout() -> u64 {
    std::env::var("CPL_CONFORM_CASE_TIMEOUT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(20)
}

/// `run_interp` on its own thread, `Err` once it runs past `case_timeout()`. A timed-out
/// run is abandoned, not stopped: the interpreter has no way to interrupt it, so on Linux
/// its thread drops to the lowest CPU priority and stops starving the cases still running.
pub fn run_interp_bounded(
    program: &cpl_ir::Program,
    path: &str,
    text: &str,
    input: &[u8],
) -> Result<RunOutput, String> {
    let limit = case_timeout();
    let (program, path, text, input) = (
        program.clone(),
        path.to_string(),
        text.to_string(),
        input.to_vec(),
    );
    let (tx, rx) = std::sync::mpsc::channel();
    let (tid_tx, tid_rx) = std::sync::mpsc::channel();
    std::thread::Builder::new()
        .stack_size(64 << 20)
        .spawn(move || {
            let _ = tid_tx.send(thread_id());
            let _ = tx.send(run_interp(&program, &path, &text, &input));
        })
        .map_err(|e| format!("could not start the interpreter: {e}"))?;
    match rx.recv_timeout(std::time::Duration::from_secs(limit)) {
        Ok(run) => Ok(run),
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
            if let Ok(tid) = tid_rx.recv() {
                deprioritize(tid);
            }
            Err(format!("the interpreter timed out after {limit}s"))
        }
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => panic!("the interpreter panicked"),
    }
}

#[cfg(target_os = "linux")]
fn thread_id() -> i32 {
    extern "C" {
        fn gettid() -> i32;
    }
    // SAFETY: gettid takes no arguments and cannot fail.
    unsafe { gettid() }
}

#[cfg(not(target_os = "linux"))]
fn thread_id() -> i32 {
    0
}

/// Sets thread `tid` to nice 19; on Linux, `PRIO_PROCESS` with a thread id names one thread.
#[cfg(target_os = "linux")]
fn deprioritize(tid: i32) {
    extern "C" {
        fn setpriority(which: i32, who: u32, prio: i32) -> i32;
    }
    // SAFETY: plain syscall wrapper; a stale tid makes it return an error, which is ignored.
    unsafe {
        setpriority(0, tid as u32, 19);
    }
}

#[cfg(not(target_os = "linux"))]
fn deprioritize(_tid: i32) {}

pub fn render_trap(path: &str, text: &str, trap: &cpl_interp::Trap) -> String {
    let lines = LineIndex::new(text);
    let diagnostic = trap.to_diagnostic(path, &lines);
    cpl_diag::render(&diagnostic, path, text, &lines)
}

/// Walks `dir` recursively, returning sorted paths whose extension equals `ext`.
pub fn walk_files(dir: &Path, ext: &str) -> Vec<PathBuf> {
    let mut out = Vec::new();
    walk_rec(dir, ext, &mut out);
    out.sort();
    out
}

fn walk_rec(dir: &Path, ext: &str, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_rec(&path, ext, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some(ext) {
            out.push(path);
        }
    }
}

/// Runs `work` over `items` in parallel across worker threads, giving each worker
/// its own `cpl_check::Database` so stdlib is analyzed once per thread.
/// Wall time of every case `parallel_with_db` has run, for the slow-case report.
pub static CASE_SECONDS: Mutex<Vec<(String, f64)>> = Mutex::new(Vec::new());

/// The `n` slowest cases run so far, slowest first.
pub fn slowest_cases(n: usize) -> Vec<(String, f64)> {
    let mut all = CASE_SECONDS.lock().unwrap().clone();
    all.sort_by(|a, b| b.1.total_cmp(&a.1));
    all.truncate(n);
    all
}

pub fn parallel_with_db<T, R, F>(items: &[T], work: F) -> BTreeMap<String, R>
where
    T: Sync,
    R: Send,
    F: Fn(&mut Database, &T) -> (String, R) + Sync,
{
    let workers = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .min(items.len().max(1));
    let next = AtomicUsize::new(0);
    let results: Mutex<BTreeMap<String, R>> = Mutex::new(BTreeMap::new());
    std::thread::scope(|s| {
        for _ in 0..workers {
            s.spawn(|| {
                let mut db = Database::new();
                loop {
                    let idx = next.fetch_add(1, Ordering::Relaxed);
                    if idx >= items.len() {
                        break;
                    }
                    let t0 = Instant::now();
                    let (key, val) = work(&mut db, &items[idx]);
                    let secs = t0.elapsed().as_secs_f64();
                    CASE_SECONDS.lock().unwrap().push((key.clone(), secs));
                    results.lock().unwrap().insert(key, val);
                }
            });
        }
    });
    results.into_inner().unwrap()
}

pub fn first_line_trimmed(s: &str, max_len: usize) -> String {
    let line = s.trim().lines().next().unwrap_or("");
    truncate_chars(line, max_len)
}

pub fn truncate_chars(s: &str, max_len: usize) -> String {
    let mut chars = s.chars();
    let out: String = chars.by_ref().take(max_len).collect();
    out
}

#[cfg(all(test, target_os = "linux"))]
mod priority_tests {
    #[test]
    fn a_deprioritized_thread_runs_at_nice_19() {
        let (tid_tx, tid_rx) = std::sync::mpsc::channel();
        let (done_tx, done_rx) = std::sync::mpsc::channel::<()>();
        let handle = std::thread::spawn(move || {
            tid_tx.send(super::thread_id()).unwrap();
            done_rx.recv().unwrap();
        });
        let tid = tid_rx.recv().unwrap();
        super::deprioritize(tid);
        let stat = std::fs::read_to_string(format!("/proc/self/task/{tid}/stat")).unwrap();
        let fields: Vec<&str> = stat
            .rsplit_once(')')
            .unwrap()
            .1
            .split_whitespace()
            .collect();
        assert_eq!(fields[16], "19", "nice field of {stat}");
        done_tx.send(()).unwrap();
        handle.join().unwrap();
    }
}
