use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

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
                    let (key, val) = work(&mut db, &items[idx]);
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
