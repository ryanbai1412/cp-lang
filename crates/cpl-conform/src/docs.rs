use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use cpl_check::Session;
use cpl_syntax::doc::{self, DocText};

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
    let mut block: Vec<&str> = Vec::new();
    let mut container = String::new();
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut push = |id: &dyn Fn(&str) -> String, block: &mut Vec<&str>| {
        let doc = DocText::parse(&block.join("\n"));
        block.clear();
        for (k, e) in doc.examples.into_iter().enumerate() {
            let body = e.text.strip_suffix('\n').unwrap_or(&e.text).to_string();
            out.push((id(&(k + 1).to_string()), body, e.block));
        }
        for (label, text) in [("empty", &doc.facts.empty), ("ties", &doc.facts.ties)] {
            if let Some(text) = text.as_deref().filter(|t| runnable_label(t)) {
                out.push((id(label), text.to_string(), false));
            }
        }
    };
    for line in text.lines() {
        if let Some(doc) = doc::doc_line(line) {
            block.push(doc);
            continue;
        }
        let trimmed = line.trim_start();
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
        if block.is_empty() {
            continue;
        }
        let occ = seen.entry(qualified.clone()).or_insert(0);
        *occ += 1;
        let item = if *occ == 1 {
            qualified
        } else {
            format!("{qualified}~{occ}")
        };
        push(&|k| format!("{rel}#{item}:{k}"), &mut block);
    }
    push(&|k| format!("{rel}#eof:{k}"), &mut block);
    out
}

/// An `empty:` or `ties:` label that is a claim `docs` runs: backticked code whose last
/// statement is `a == b`, `a != b` or `not a`, or code tagged `(traps R0401)` / `(error E0302)`.
pub fn runnable_label(text: &str) -> bool {
    let e = doc::Example::from_line(text);
    let Some(code) = e.code.as_deref() else {
        return false;
    };
    let last = doc::statements(code, false).pop().unwrap_or_default();
    e.expect.is_some() || e.claim().2.is_some() || last.contains(" != ") || last.starts_with("not ")
}

/// The name an item header declares: `pub fn up() -> Dir4` is `up`,
/// `impl Monoid for Sum:` is `Monoid for Sum`.
fn item_name(header: &str) -> String {
    let mut rest = header.strip_prefix("pub ").unwrap_or(header).trim_start();
    if let Some(after) = rest.strip_prefix("impl") {
        let end = after.find([':', '=']).unwrap_or(after.len());
        return after[..end].trim().to_string();
    }
    if let Some((first, after)) = rest.split_once(' ') {
        // A field, `src: S,`, names itself; other headers lead with a keyword.
        if let Some(field) = first.strip_suffix(':') {
            return field.to_string();
        }
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
/// trap or error code (`cpl_syntax::doc` reads it); `None` code when it is prose.
pub fn parse_example_body(body: &str) -> (Option<String>, String, Option<String>) {
    let e = doc::Example::from_line(body);
    let stdin = e
        .stdin
        .filter(|s| !s.is_empty())
        .map(|s| s + "\n")
        .unwrap_or_default();
    (e.code, stdin, e.expect)
}

fn build_example_program(ex: &DocExample, code: &str, import_module: bool) -> String {
    let mut lines = Vec::new();
    if import_module && !ex.module.is_empty() && ex.module != "prelude" && ex.module != "core" {
        lines.push(format!("use std.{}.*", ex.module));
    }
    let stmts = doc::statements(code, ex.block);
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
    let mut outcomes = outcomes;
    native_labels(root, &examples, &mut outcomes);
    SuiteReport::from_outcomes(outcomes)
}

/// The `empty:`/`ties:` claims that passed in the interpreter run again in the C++ build, one
/// program per std module, so an edge holds on both engines (std-docs/R2). A module whose
/// program does not build or traps runs its claims one by one to name the failing one.
fn native_labels(
    root: &Path,
    examples: &[DocExample],
    outcomes: &mut BTreeMap<String, CaseOutcome>,
) {
    let mut by_module: BTreeMap<&str, Vec<&DocExample>> = BTreeMap::new();
    for ex in examples {
        let label = ex.id.ends_with(":empty") || ex.id.ends_with(":ties");
        let passed = outcomes.get(&ex.id).is_some_and(|o| o.status == "pass");
        if label && passed && ex.expect.is_none() && ex.snippet.is_some() {
            by_module.entry(ex.module.as_str()).or_default().push(ex);
        }
    }
    let groups: Vec<Vec<&DocExample>> = by_module.into_values().collect();
    let failed = runner::parallel_with_db(&groups, |db, group| {
        let key = group.first().map_or_else(String::new, |ex| ex.id.clone());
        let whole = [claims_program(group, false), claims_program(group, true)]
            .into_iter()
            .any(|src| native_run(db, root, &key, &src).is_ok());
        let fails: Vec<(String, CaseOutcome)> = if whole {
            Vec::new()
        } else {
            group
                .iter()
                .filter_map(|ex| {
                    let one = std::slice::from_ref(ex);
                    let mut last = String::new();
                    for src in [claims_program(one, false), claims_program(one, true)] {
                        match native_run(db, root, &ex.id, &src) {
                            Ok(()) => return None,
                            Err(e) => last = e,
                        }
                    }
                    Some((
                        ex.id.clone(),
                        CaseOutcome {
                            status: "fail".into(),
                            detail: format!(
                                "passes in the interpreter, not in the C++ build: {last}"
                            ),
                        },
                    ))
                })
                .collect()
        };
        (key, fails)
    });
    for (id, outcome) in failed.into_values().flatten() {
        outcomes.insert(id, outcome);
    }
}

/// One program asserting every claim of `group`, each in its own fn so their names stay apart.
fn claims_program(group: &[&DocExample], import_module: bool) -> String {
    let mut lines = Vec::new();
    let module = group.first().map_or("", |ex| ex.module.as_str());
    if import_module && !module.is_empty() && module != "prelude" && module != "core" {
        lines.push(format!("use std.{module}.*"));
    }
    for (k, ex) in group.iter().enumerate() {
        let code = ex.snippet.as_deref().unwrap_or_default();
        lines.push(format!("fn claim_{k}():"));
        for line in build_example_program(ex, code, false).lines() {
            lines.push(format!("    {line}"));
        }
    }
    lines.extend((0..group.len()).map(|k| format!("claim_{k}()")));
    lines.join("\n") + "\n"
}

/// Builds `source` to C++ and runs it; `Err` says why it did not build or finish.
fn native_run(db: &mut Session, root: &Path, id: &str, source: &str) -> Result<(), String> {
    let safe: String = id
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    let path = root
        .join(format!("__cpl_conform_doc_native_{safe}.cp"))
        .to_string_lossy()
        .to_string();
    let compiled = runner::compile_program(db, &path, source);
    db.remove_file(&path);
    let compiled =
        compiled.map_err(|CompileError::Diagnostics(d)| runner::first_line_trimmed(&d, 160))?;
    let cpp = cpl_codegen::emit(&compiled.program, &path, Some(source));
    let cache = crate::conformance::cxx_cache("docs");
    let bin = crate::diff::compile_cpp(&cpp, &cache).map_err(|e| match e {
        crate::diff::CppCompileError::Cxx(m) | crate::diff::CppCompileError::Infra(m) => m,
    })?;
    let out = runner::run_piped(std::process::Command::new(bin), b"", 10)?;
    if out.status.success() {
        Ok(())
    } else {
        Err(runner::first_line_trimmed(
            &String::from_utf8_lossy(&out.stderr),
            160,
        ))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_field_doc_example_is_named_by_the_field() {
        let text =
            "pub struct View<S, T>(\n    /// example: `1 == 1`\n    pub src: S,\n    p: T,\n)\n";
        let ids: Vec<_> = super::doc_examples("seq.cp", text)
            .into_iter()
            .map(|(id, _, _)| id)
            .collect();
        assert_eq!(ids, ["seq.cp#View.src:1"]);
    }

    #[test]
    fn a_claim_edge_label_runs_and_a_value_or_prose_does_not() {
        let text = "/// The first.\n///\n/// empty: `SortedBag<Int>().first == None`\n\
                    /// ties: the leftmost\npub fn first(self) -> ?K = todo\n\
                    /// empty: `None`\npub fn last(self) -> ?K = todo\n";
        let got: Vec<_> = super::doc_examples("bag.cp", text)
            .into_iter()
            .map(|(id, body, _)| (id, body))
            .collect();
        assert_eq!(
            got,
            [(
                "bag.cp#first:empty".to_string(),
                "`SortedBag<Int>().first == None`".to_string()
            )]
        );
        let docs: Vec<super::DocExample> = ["`[1].len == 1`", "`let a = [2]; a.first == Some(2)`"]
            .into_iter()
            .enumerate()
            .map(|(k, code)| super::DocExample {
                id: format!("seq.cp#f:{k}"),
                module: "seq".into(),
                snippet: Some(code.trim_matches('`').into()),
                stdin: String::new(),
                block: false,
                expect: None,
            })
            .collect();
        let group: Vec<_> = docs.iter().collect();
        assert_eq!(
            super::claims_program(&group, true),
            "use std.seq.*\nfn claim_0():\n    assert([1].len == 1)\nfn claim_1():\n    \
             let a = [2]\n    assert(a.first == Some(2))\nclaim_0()\nclaim_1()\n"
        );
        assert!(super::runnable_label("`[].max()` (traps R0401)"));
        assert!(!super::runnable_label("`0`"));
    }
}
