use std::collections::HashMap;
use std::path::Path;

use cpl_check::Session;

use crate::runner::{self, CompileError};
use crate::scoreboard::{CaseOutcome, SuiteReport};

#[derive(Clone, Debug)]
pub struct DocExample {
    pub id: String,
    pub module: String,
    pub snippet: Option<String>,
    pub stdin: String,
    /// A fenced multi-line block rather than an `example:` line.
    pub block: bool,
    /// The diagnostic or trap code a `(error E0302)` / `(traps R0401)` tag says the example
    /// stops with; `None` means it must run to the end with its assertion true.
    pub expect: Option<String>,
}

/// The `/// example:` lines of one std file, as `(id, body)`. The id is
/// `file#item:k`, the k-th example in the doc comment of `item` (`Type.item`
/// inside a type's block), so editing an example keeps its id. A repeated item
/// name gets `~2`, `~3`, ... in file order.
pub fn example_ids(rel: &str, text: &str) -> Vec<(String, String)> {
    doc_examples(rel, text)
        .into_iter()
        .map(|(id, body, _)| (id, body))
        .collect()
}

/// Like [`example_ids`], also counting fenced ```` ``` ```` / ```` ```cp ```` blocks in
/// `///` comments, as `(id, body, block)`. A block's body is its code, dedented.
pub fn doc_examples(rel: &str, text: &str) -> Vec<(String, String, bool)> {
    let mut out = Vec::new();
    let mut pending: Vec<(String, bool)> = Vec::new();
    let mut container = String::new();
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut fence: Option<(usize, Vec<String>)> = None;
    for line in text.lines() {
        let trimmed = line.trim_start();
        if let Some(after_doc) = trimmed.strip_prefix("///") {
            let doc_line = after_doc.strip_prefix(' ').unwrap_or(after_doc);
            let marker = doc_line.trim();
            match &mut fence {
                Some((indent, lines)) if marker == "```" => {
                    let indent = *indent;
                    let code: Vec<String> = lines
                        .drain(..)
                        .map(|l| l.get(indent..).unwrap_or(l.trim_start()).to_string())
                        .collect();
                    fence = None;
                    pending.push((code.join("\n"), true));
                }
                Some((_, lines)) => lines.push(doc_line.to_string()),
                None if marker == "```" || marker == "```cp" => {
                    fence = Some((doc_line.len() - doc_line.trim_start().len(), Vec::new()));
                }
                None => {
                    if let Some(rest) = doc_line.trim_start().strip_prefix("example:") {
                        pending.push((rest.trim().to_string(), false));
                    }
                }
            }
            continue;
        }
        fence = None;
        if trimmed.is_empty() || trimmed.starts_with('@') || trimmed.starts_with("//") {
            continue;
        }
        let name = item_name(trimmed);
        let top = trimmed.len() == line.len();
        let qualified = if top || container.is_empty() {
            name.clone()
        } else {
            format!("{container}.{name}")
        };
        if top {
            container = name;
        }
        if pending.is_empty() {
            continue;
        }
        let occ = seen.entry(qualified.clone()).or_insert(0);
        *occ += 1;
        let item = if *occ == 1 {
            qualified
        } else {
            format!("{qualified}~{occ}")
        };
        for (k, (body, block)) in pending.drain(..).enumerate() {
            out.push((format!("{rel}#{item}:{}", k + 1), body, block));
        }
    }
    for (k, (body, block)) in pending.into_iter().enumerate() {
        out.push((format!("{rel}#eof:{}", k + 1), body, block));
    }
    out
}

/// The name an item header declares: `pub fn up() -> Dir4` is `up`,
/// `impl Monoid for Sum:` is `Monoid for Sum`.
fn item_name(header: &str) -> String {
    let mut rest = header.strip_prefix("pub ").unwrap_or(header).trim_start();
    if let Some(after) = rest.strip_prefix("impl") {
        let end = after.find([':', '=']).unwrap_or(after.len());
        return after[..end].trim().to_string();
    }
    if let Some((_keyword, after)) = rest.split_once(' ') {
        rest = after.trim_start();
    }
    let end = rest
        .find(|c: char| !(c.is_alphanumeric() || c == '_'))
        .unwrap_or(rest.len());
    rest[..end].to_string()
}

/// Discovers `/// example:` lines in `std/*.cp`.
pub fn discover(root: &Path, filter: Option<&str>) -> Vec<DocExample> {
    let dir = root.join("std");
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
        for (id, body, block) in doc_examples(&rel, &text) {
            if let Some(f) = filter {
                if !id.contains(f) && !body.contains(f) {
                    continue;
                }
            }
            let (snippet, stdin, expect) = if block {
                (Some(body), String::new(), None)
            } else {
                parse_example_body(&body)
            };
            out.push(DocExample {
                id,
                module: module.clone(),
                snippet,
                stdin,
                block,
                expect,
            });
        }
    }
    out
}

/// Splits `` `code` (stdin `x`) (traps R0401) `` into the code, its stdin and the expected
/// trap or error code. The tags may come in any order; anything else after the code is not
/// an example (`None` code).
pub fn parse_example_body(body: &str) -> (Option<String>, String, Option<String>) {
    let none = (None, String::new(), None);
    let Some(after_open) = body.strip_prefix('`') else {
        return none;
    };
    let Some(end) = after_open.find('`') else {
        return none;
    };
    let code = after_open[..end].to_string();
    let mut tail = after_open[end + 1..].trim();
    let mut stdin = String::new();
    let mut expect = None;
    while !tail.is_empty() {
        if let Some(rest) = tail.strip_prefix("(stdin `") {
            let Some(close) = rest.find("`)") else {
                return none;
            };
            stdin = format!("{}\n", &rest[..close]);
            tail = rest[close + 2..].trim_start();
        } else if let Some(rest) = tail
            .strip_prefix("(traps ")
            .or_else(|| tail.strip_prefix("(error "))
        {
            let Some(close) = rest.find(')') else {
                return none;
            };
            expect = Some(rest[..close].trim().to_string());
            tail = rest[close + 1..].trim_start();
        } else {
            return none;
        }
    }
    (Some(code), stdin, expect)
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

/// Splits a fenced block into its top-level statements: a line that is indented or starts by
/// closing a bracket continues the statement above, so a multi-line call stays one statement.
fn split_top_level_lines(code: &str) -> Vec<String> {
    let mut parts: Vec<String> = Vec::new();
    for line in code.lines() {
        if line.trim().is_empty() || line.trim_start().starts_with("//") {
            continue;
        }
        let continues = line.starts_with([' ', '\t', ')', ']', '}']);
        match parts.last_mut() {
            Some(last) if continues => {
                last.push('\n');
                last.push_str(line);
            }
            _ => parts.push(line.to_string()),
        }
    }
    parts
}

fn build_example_program(ex: &DocExample, code: &str, import_module: bool) -> String {
    let mut lines = Vec::new();
    if import_module && !ex.module.is_empty() && ex.module != "prelude" && ex.module != "core" {
        lines.push(format!("use std.{}.*", ex.module));
    }
    let stmts = if ex.block {
        split_top_level_lines(code)
    } else {
        split_top_level_semis(code)
    };
    if let Some((last, init)) = stmts.split_last() {
        for s in init {
            lines.push(s.clone());
        }
        const STATEMENTS: [&str; 11] = [
            "let ", "var ", "for ", "if ", "while ", "print", "assert", "domain ", "fn ", "use ",
            "const ",
        ];
        if ex.block && STATEMENTS.iter().any(|k| last.starts_with(k)) {
            lines.push(last.clone());
        } else {
            lines.push(format!("assert({last})"));
        }
    }
    let mut prog = lines.join("\n");
    prog.push('\n');
    prog
}

pub fn run_one(db: &mut Session, root: &Path, ex: &DocExample) -> CaseOutcome {
    let Some(code) = &ex.snippet else {
        return CaseOutcome {
            status: "prose".into(),
            detail: "`example:` must be backticked code, optionally followed by `(stdin `...`)`, \
                     `(traps RXXXX)` or `(error EXXXX)`; move prose to the doc text"
                .into(),
        };
    };
    if ex.expect.as_deref() == Some("R0402") {
        return CaseOutcome {
            status: "bad".into(),
            detail: "a `todo` body is unfinished, not an expected trap: write the body".into(),
        };
    }
    let outcome = run_unchecked(db, root, ex, code);
    match &ex.expect {
        None => outcome,
        Some(want) => {
            let stopped = outcome.status == "error" || outcome.status == "trap";
            if stopped && outcome.detail.contains(&format!("[{want}]")) {
                CaseOutcome {
                    status: "pass".into(),
                    detail: String::new(),
                }
            } else {
                CaseOutcome {
                    status: "fail".into(),
                    detail: format!(
                        "expected {want}, got {}: {}",
                        outcome.status, outcome.detail
                    ),
                }
            }
        }
    }
}

fn run_unchecked(db: &mut Session, root: &Path, ex: &DocExample, code: &str) -> CaseOutcome {
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
        // Any failure retries with the module imported: its own names (`poly.sqrt`) may be
        // shadowed by a prelude name of another arity rather than unresolved.
        Err(CompileError::Diagnostics(_))
            if !ex.module.is_empty() && ex.module != "prelude" && ex.module != "core" =>
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
    let run = match runner::run_interp_bounded(
        &compiled.program,
        &path_str,
        &source,
        ex.stdin.as_bytes(),
    ) {
        Ok(run) => run,
        Err(detail) => {
            return CaseOutcome {
                status: "error".into(),
                detail,
            }
        }
    };
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
