//! Every syntax form reaches every tool's suite.
//!
//! A feature isn't done until it works in every tool, and a tool's suite can
//! only show that for forms its programs contain. This walks the programs
//! each suite runs, collects the statement and expression forms they use,
//! and fails on any form some suite uses that another tool's suite does not.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use cpl_syntax::ast::{Expr, ExprKind, Stmt, StmtKind};
use cpl_syntax::visit::{walk_stmt, Visitor};

/// Each tool and the programs its suite runs it on; `@corpus` is every problem
/// package, `@corpus/<leaf>` each package's leaf file.
const SUITES: [(&str, &str); 5] = [
    // `cpl-conform diff`: conformance cases and corpus ports.
    ("cpp", "spec/conformance @corpus"),
    // `every_corpus_program_and_conformance_case_is_served`.
    ("lsp", "spec/conformance @corpus"),
    // `scripts/fmt_check.py`: every tracked `.cp` file.
    ("fmt", "spec corpus std crates/cpl-cli/tests"),
    // The saved-trace programs, traced and untraced.
    ("trace", "crates/cpl-cli/tests/programs"),
    // Lint records, `std/` and the corpus specs.
    ("lint", "spec/lints std @corpus/spec.cp"),
];

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn cp_files(dir: &Path, leaf: Option<&str>, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries {
        let path = entry.unwrap().path();
        if path.is_dir() {
            cp_files(&path, leaf, out);
        } else if path.extension().is_some_and(|e| e == "cp")
            && leaf.is_none_or(|l| path.file_name().is_some_and(|n| n == l))
        {
            out.push(path);
        }
    }
}

fn suite_files(dirs: &str) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for dir in dirs.split(' ') {
        if let Some(rest) = dir.strip_prefix("@corpus") {
            match rest.strip_prefix('/') {
                Some(leaf) => out.extend(cpl_corpus::files(&root(), leaf)),
                None => {
                    for package in cpl_corpus::packages(&root()) {
                        cp_files(&package, None, &mut out);
                    }
                }
            }
            continue;
        }
        match dir.split_once("/*/") {
            Some((parent, leaf)) => cp_files(&root().join(parent), Some(leaf), &mut out),
            None => cp_files(&root().join(dir), None, &mut out),
        }
    }
    out
}

fn variant(debug: String) -> String {
    debug
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric())
        .collect()
}

#[derive(Default)]
struct Forms(BTreeSet<String>);

impl Visitor for Forms {
    fn stmt(&mut self, stmt: &Stmt) {
        if !matches!(stmt.kind, StmtKind::Error) {
            self.0
                .insert(format!("stmt {}", variant(format!("{:?}", stmt.kind))));
        }
    }
    fn expr(&mut self, expr: &Expr) {
        if !matches!(expr.kind, ExprKind::Error | ExprKind::Missing) {
            self.0
                .insert(format!("expr {}", variant(format!("{:?}", expr.kind))));
        }
    }
}

fn forms_in(files: &[PathBuf]) -> BTreeSet<String> {
    let mut forms = Forms::default();
    for file in files {
        let Ok(src) = std::fs::read_to_string(file) else {
            continue;
        };
        for (_, item) in cpl_syntax::parse_file(&src).items {
            for stmt in &item.stmts {
                walk_stmt(stmt, &mut forms);
            }
        }
    }
    forms.0
}

/// Enforces `compiler-changes/R12` (handbook/guidelines/compiler-changes-checklist.md).
#[test]
fn every_syntax_form_reaches_every_tools_suite() {
    let per_tool: BTreeMap<&str, BTreeSet<String>> = SUITES
        .iter()
        .map(|&(tool, dirs)| (tool, forms_in(&suite_files(dirs))))
        .collect();
    let all: BTreeSet<&String> = per_tool.values().flatten().collect();
    let gaps: BTreeSet<String> = per_tool
        .iter()
        .flat_map(|(tool, forms)| {
            all.iter()
                .filter(|f| !forms.contains(**f))
                .map(move |f| format!("{tool} {f}"))
        })
        .collect();
    assert!(
        gaps.is_empty(),
        "Forms a tool's suite never uses (add a program using the form to that tool's suite):\n  {}",
        gaps.iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join("\n  "),
    );
}
