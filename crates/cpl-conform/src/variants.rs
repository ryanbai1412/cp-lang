//! Side-effect variants of corpus ports for `cpl-conform diff`.
//!
//! Each top-level `var` of a `port.cp` gets probes that make mutation observable:
//! prints of its state after every top-level loop, writes through a `mut`
//! parameter, a closure capture and a `mut` place, and writes to a copy that
//! must leave the original alone. Probes the checker rejects (wrong type, a
//! moved value) are dropped, so what remains is a valid program whose output
//! the interpreter and the C++ must agree on.

use cpl_check::Database;
use cpl_diag::{LineIndex, Severity};

/// Lines inserted before line `at` of the port, or appended when `at` is `None`.
#[derive(Clone, Debug, PartialEq)]
pub struct Probe {
    pub at: Option<usize>,
    pub lines: Vec<String>,
}

const MAX_PROBES: usize = 48;

fn ident_at(s: &str) -> Option<&str> {
    let end = s
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .unwrap_or(s.len());
    let name = &s[..end];
    (!name.is_empty() && !name.starts_with(|c: char| c.is_ascii_digit())).then_some(name)
}

/// Every probe for `text`, in program order.
pub fn probes(text: &str) -> Vec<Probe> {
    let lines: Vec<&str> = text.lines().collect();
    let top = |l: &str| !l.is_empty() && !l.starts_with([' ', '\t']) && !l.starts_with("//");
    let mut vars: Vec<String> = Vec::new();
    let mut out = Vec::new();
    let mut in_loop = false;
    for (i, line) in lines.iter().enumerate() {
        if !top(line) {
            continue;
        }
        let resumes = !line.starts_with(|c: char| ")]}.".contains(c))
            && !line.starts_with("else")
            && !line.starts_with("elif");
        if in_loop && resumes {
            out.extend(vars.iter().map(|v| Probe {
                at: Some(i),
                lines: vec![format!("print({v})")],
            }));
        }
        in_loop = line.starts_with("for ") || line.starts_with("while ");
        if let Some(v) = line.strip_prefix("var ").and_then(ident_at) {
            if !vars.iter().any(|x| x == v) {
                vars.push(v.to_string());
            }
        }
    }
    for (k, v) in vars.iter().enumerate() {
        let end = |ls: &[String]| Probe {
            at: None,
            lines: ls.to_vec(),
        };
        let p = format!("cpl_probe{k}");
        out.push(end(&[format!("print({v})")]));
        // Plain values: a write through `mut`, a capture, a place, a copy.
        out.push(end(&[
            format!("fn {p}_bump(mut x: Int): x += 1"),
            format!("{p}_bump(mut {v})"),
            format!("print({v})"),
        ]));
        out.push(end(&[
            format!("let {p}_inc = () => {v} += 2"),
            format!("{p}_inc()"),
            format!("{p}_inc()"),
            format!("print({v})"),
        ]));
        out.push(end(&[
            format!("let {p}_place = mut {v}"),
            format!("{p}_place += 5"),
            format!("print({v})"),
        ]));
        out.push(end(&[
            format!("var {p}_copy = {v}"),
            format!("{p}_copy += 3"),
            format!("print({v})"),
            format!("print({p}_copy)"),
        ]));
        // Sequences: the same writes as an in-place reverse.
        out.push(end(&[
            format!("fn {p}_rev<S: IndexedMut>(mut xs: S): xs.reverse()"),
            format!("{p}_rev(mut {v})"),
            format!("print({v})"),
        ]));
        out.push(end(&[
            format!("let {p}_flip = () => {v}.reverse()"),
            format!("{p}_flip()"),
            format!("print({v})"),
        ]));
        out.push(end(&[
            format!("var {p}_dup = {v}.clone()"),
            format!("{p}_dup.reverse()"),
            format!("print({v})"),
            format!("print({p}_dup)"),
        ]));
        // Containers: clearing a copy leaves the original's size.
        out.push(end(&[
            format!("var {p}_empty = {v}.clone()"),
            format!("{p}_empty.clear()"),
            format!("print({v}.len)"),
            format!("print({p}_empty.len)"),
        ]));
    }
    out.truncate(MAX_PROBES);
    out
}

/// `text` with the `keep` probes inserted, and for each output line the probe it came from.
pub fn apply(text: &str, probes: &[Probe], keep: &[bool]) -> (String, Vec<Option<usize>>) {
    let mut out = String::new();
    let mut owner = Vec::new();
    let mut push = |s: &str, who: Option<usize>, out: &mut String| {
        out.push_str(s);
        out.push('\n');
        owner.push(who);
    };
    let lines: Vec<&str> = text.lines().collect();
    for i in 0..=lines.len() {
        let at = (i < lines.len()).then_some(i);
        for (j, p) in probes.iter().enumerate() {
            if keep[j] && p.at == at {
                for l in &p.lines {
                    push(l, Some(j), &mut out);
                }
            }
        }
        if let Some(l) = lines.get(i) {
            push(l, None, &mut out);
        }
    }
    (out, owner)
}

/// Zero-based lines of the error diagnostics `text` gets from the checker and lowering.
fn error_lines(db: &mut Database, path: &str, text: &str) -> Option<Vec<usize>> {
    db.set_file(path, text);
    let analysis = db.analyze(path).expect("the file was just set");
    db.remove_file(path);
    let errors: Vec<_> = analysis
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .cloned()
        .collect();
    let errors = if errors.is_empty() {
        match cpl_ir::lower(&analysis) {
            Ok(_) => return None,
            Err(diags) => diags,
        }
    } else {
        errors
    };
    let idx = LineIndex::new(text);
    Some(
        errors
            .iter()
            .map(|d| idx.line_of(d.span.start) as usize)
            .collect(),
    )
}

/// The variant of `text` with every probe that checks, and how many probes it kept;
/// `None` when no probe survives.
pub fn build(db: &mut Database, path: &str, text: &str) -> Option<(String, usize)> {
    let probes = probes(text);
    let mut keep = vec![true; probes.len()];
    // Drop the probes the errors point at; one pass per layer of errors.
    for _ in 0..8 {
        if !keep.contains(&true) {
            return None;
        }
        let (variant, owner) = apply(text, &probes, &keep);
        let Some(lines) = error_lines(db, path, &variant) else {
            return Some((variant, keep.iter().filter(|k| **k).count()));
        };
        let blamed: Vec<usize> = lines
            .iter()
            .filter_map(|l| owner.get(*l).copied().flatten())
            .collect();
        if blamed.is_empty() {
            break;
        }
        for j in blamed {
            keep[j] = false;
        }
    }
    // An error no probe line owns: add the probes one at a time.
    let mut keep = vec![false; probes.len()];
    for j in 0..probes.len() {
        keep[j] = true;
        let (variant, _) = apply(text, &probes, &keep);
        if error_lines(db, path, &variant).is_some() {
            keep[j] = false;
        }
    }
    let n = keep.iter().filter(|k| **k).count();
    (n > 0).then(|| (apply(text, &probes, &keep).0, n))
}

#[cfg(test)]
#[path = "variants_tests.rs"]
mod tests;
