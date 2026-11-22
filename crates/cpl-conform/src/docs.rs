use std::path::Path;

use cpl_check::Database;

use crate::runner::{self, CompileError};
use crate::scoreboard::{CaseOutcome, SuiteReport};

#[derive(Clone, Debug)]
pub struct DocExample {
    pub id: String,
    pub module: String,
    pub snippet: Option<String>,
    pub stdin: String,
}

/// Discovers `/// example:` lines in `spec/std/*.cp`.
pub fn discover(root: &Path, filter: Option<&str>) -> Vec<DocExample> {
    let dir = root.join("spec").join("std");
    let mut out = Vec::new();
    for path in runner::walk_files(&dir, "cp") {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let rel = path
            .strip_prefix(&dir)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        let module = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        for (idx, line) in text.lines().enumerate() {
            let trimmed = line.trim_start();
            let Some(after_doc) = trimmed.strip_prefix("///") else {
                continue;
            };
            let Some(rest) = after_doc.trim_start().strip_prefix("example:") else {
                continue;
            };
            let id = format!("{rel}:{}", idx + 1);
            if let Some(f) = filter {
                if !id.contains(f) {
                    continue;
                }
            }
            let (snippet, stdin) = parse_example_body(rest.trim());
            out.push(DocExample {
                id,
                module: module.clone(),
                snippet,
                stdin,
            });
        }
    }
    out
}

fn parse_example_body(body: &str) -> (Option<String>, String) {
    let Some(after_open) = body.strip_prefix('`') else {
        return (None, String::new());
    };
    let Some(end) = after_open.find('`') else {
        return (None, String::new());
    };
    let code = after_open[..end].to_string();
    let tail = after_open[end + 1..].trim();
    let mut stdin = String::new();
    if let Some(inner) = tail
        .strip_prefix("(stdin `")
        .and_then(|s| s.strip_suffix("`)"))
    {
        stdin = inner.to_string();
        stdin.push('\n');
    }
    (Some(code), stdin)
}

/// Splits a semicolon-separated example snippet at top level (ignoring `;` inside
/// strings, char literals, or brackets/parens/braces).
fn split_top_level_semis(code: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut cur = String::new();
    let mut depth: usize = 0;
    let mut in_str = false;
    let mut in_char = false;
    let mut escape = false;

    for ch in code.chars() {
        if escape {
            cur.push(ch);
            escape = false;
            continue;
        }
        if in_str {
            cur.push(ch);
            if ch == '\\' {
                escape = true;
            } else if ch == '"' {
                in_str = false;
            }
            continue;
        }
        if in_char {
            cur.push(ch);
            if ch == '\\' {
                escape = true;
            } else if ch == '\'' {
                in_char = false;
            }
            continue;
        }
        match ch {
            '"' => {
                in_str = true;
                cur.push(ch);
            }
            '\'' => {
                in_char = true;
                cur.push(ch);
            }
            '(' | '[' | '{' => {
                depth += 1;
                cur.push(ch);
            }
            ')' | ']' | '}' => {
                depth = depth.saturating_sub(1);
                cur.push(ch);
            }
            ';' if depth == 0 => {
                let s = cur.trim();
                if !s.is_empty() {
                    parts.push(s.to_string());
                }
                cur.clear();
            }
            _ => cur.push(ch),
        }
    }
    let s = cur.trim();
    if !s.is_empty() {
        parts.push(s.to_string());
    }
    parts
}

fn build_example_program(ex: &DocExample, code: &str, import_module: bool) -> String {
    let mut lines = Vec::new();
    if import_module && !ex.module.is_empty() && ex.module != "prelude" && ex.module != "core" {
        lines.push(format!("use std.{}.*", ex.module));
    }
    let stmts = split_top_level_semis(code);
    if let Some((last, init)) = stmts.split_last() {
        for s in init {
            lines.push(s.clone());
        }
        lines.push(format!("assert({last})"));
    }
    let mut prog = lines.join("\n");
    prog.push('\n');
    prog
}

pub fn run_one(db: &mut Database, root: &Path, ex: &DocExample) -> CaseOutcome {
    let Some(code) = &ex.snippet else {
        return CaseOutcome {
            status: "skip".into(),
            detail: "prose example".into(),
        };
    };
    let safe_id: String = ex
        .id
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    let virtual_path = root.join(format!("__cpl_conform_doc_{safe_id}.cp"));
    let path_str = virtual_path.to_string_lossy().to_string();
    let mut source = build_example_program(ex, code, false);
    let compiled = match runner::compile_program(db, &path_str, &source) {
        Ok(c) => {
            db.remove_file(&path_str);
            c
        }
        Err(CompileError::Diagnostics(d))
            if (d.contains("E0201") || d.contains("E0204"))
                && !ex.module.is_empty()
                && ex.module != "prelude"
                && ex.module != "core" =>
        {
            db.remove_file(&path_str);
            let path_str2 = root
                .join(format!("__cpl_conform_doc_{safe_id}_use.cp"))
                .to_string_lossy()
                .to_string();
            source = build_example_program(ex, code, true);
            let res = runner::compile_program(db, &path_str2, &source);
            db.remove_file(&path_str2);
            match res {
                Ok(c) => c,
                Err(CompileError::Diagnostics(d2)) => {
                    return CaseOutcome {
                        status: "error".into(),
                        detail: runner::first_line_trimmed(&d2, 160),
                    };
                }
            }
        }
        Err(CompileError::Diagnostics(d)) => {
            db.remove_file(&path_str);
            return CaseOutcome {
                status: "error".into(),
                detail: runner::first_line_trimmed(&d, 160),
            };
        }
    };
    let run = runner::run_interp(&compiled.program, &path_str, &source, ex.stdin.as_bytes());
    if run.trap.is_some() {
        let first = runner::first_line_trimmed(&run.trap_rendered, 160);
        let status = if first.contains("`todo` reached") {
            "todo"
        } else {
            "trap"
        };
        return CaseOutcome {
            status: status.into(),
            detail: first,
        };
    }
    CaseOutcome {
        status: "pass".into(),
        detail: String::new(),
    }
}

pub fn run(root: &Path, filter: Option<&str>) -> SuiteReport {
    let examples = discover(root, filter);
    let outcomes = runner::parallel_with_db(&examples, |db, ex| {
        let outcome =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run_one(db, root, ex)))
                .unwrap_or_else(|_| CaseOutcome {
                    status: "panic".into(),
                    detail: "interpreter/checker panicked".into(),
                });
        (ex.id.clone(), outcome)
    });
    SuiteReport::from_outcomes(outcomes)
}
