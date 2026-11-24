//! Std bounds mean what they say: for each `pub fn` in `std/` whose generics
//! bound a type by `Ord`, `Eq` or `Hash`, a call is generated that fills that
//! type with a fn value, which has none of them. Each call must be rejected
//! at the call, in the caller's file, never accepted or reported inside std.
//! Enforces the "Std bounds mean what they say" row of
//! `handbook/domains/compiler.md`.

use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

const BOUNDS: [&str; 3] = ["Ord", "Eq", "Hash"];
/// Calls generated today; the generator may only grow.
const AT_LEAST: usize = 8;

fn split_top(s: &str) -> Vec<String> {
    let (mut out, mut depth, mut cur) = (Vec::new(), 0i32, String::new());
    for c in s.chars() {
        match c {
            '<' | '(' | '[' | '{' => depth += 1,
            '>' | ')' | ']' | '}' => depth -= 1,
            _ => {}
        }
        if c == ',' && depth == 0 {
            out.push(cur.trim().to_string());
            cur.clear();
        } else {
            cur.push(c);
        }
    }
    if !cur.trim().is_empty() {
        out.push(cur.trim().to_string());
    }
    out
}

/// The text between the bracket opening at `open` and its match.
fn enclosed(s: &str, open: usize) -> Option<&str> {
    let mut depth = 0;
    for (i, c) in s[open..].char_indices() {
        match c {
            '<' | '(' | '[' => depth += 1,
            '>' | ')' | ']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&s[open + 1..open + i]);
                }
            }
            _ => {}
        }
    }
    None
}

/// Every `pub fn` signature in `text`, joined onto one line.
fn signatures(text: &str) -> Vec<String> {
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if lines[i].starts_with("pub fn ") {
            let mut sig = lines[i].to_string();
            while sig.matches('(').count() > sig.matches(')').count() && i + 1 < lines.len() {
                i += 1;
                sig.push(' ');
                sig.push_str(lines[i].trim());
            }
            out.push(sig);
        }
        i += 1;
    }
    out
}

/// A call to the fn `sig` declares that puts a fn value where a bounded type
/// goes, or `None` when the generator can't fill its other parameters.
fn violating_call(sig: &str) -> Option<(String, String)> {
    let rest = sig.strip_prefix("pub fn ")?;
    let name_end = rest.find('<')?;
    let name = &rest[..name_end];
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return None;
    }
    let gens = enclosed(rest, name_end)?;
    let after = name_end + gens.len() + 2;
    if !rest[after..].starts_with('(') {
        return None;
    }
    let params = enclosed(rest, after)?;
    let bounds: BTreeMap<String, Vec<String>> = split_top(gens)
        .into_iter()
        .map(|g| match g.split_once(':') {
            Some((n, b)) => (
                n.trim().to_string(),
                b.split('+').map(|t| t.trim().to_string()).collect(),
            ),
            None => (g.trim().to_string(), Vec::new()),
        })
        .collect();
    let bounded = |n: &str| {
        bounds.get(n).is_some_and(|b| {
            b.iter()
                .any(|t| BOUNDS.contains(&t.split('<').next().unwrap_or("")))
        })
    };
    let (mut args, mut recv, mut hit) = (Vec::new(), None, false);
    for p in split_top(params) {
        let (head, has_default) = match p.split_once(" = ") {
            Some((h, _)) => (h.trim().to_string(), true),
            None => (p.clone(), false),
        };
        let mutable = head.starts_with("mut ");
        let (pn, pt) = head.trim_start_matches("mut ").split_once(':')?;
        let (pn, pt) = (pn.trim(), pt.trim());
        let value = if bounded(pt) {
            "bad".to_string()
        } else if let Some(traits) = bounds.get(pt) {
            // `S: Seq<T>`, `S: Indexed<K, T>`, or a bare `S: Seq` beside a bounded `T`.
            let elem = traits.iter().find_map(|t| {
                let head = t.split('<').next().unwrap_or("");
                ["Seq", "Indexed", "IndexedMut", "Keyed"]
                    .contains(&head)
                    .then(|| {
                        t.find('<')
                            .and_then(|o| enclosed(t, o))
                            .and_then(|a| split_top(a).pop())
                    })
            })?;
            let bad = match elem {
                Some(e) => bounded(&e),
                None => bounds.keys().any(|k| k != pt && bounded(k)),
            };
            if bad { "[bad, bad]" } else { "[1, 2]" }.to_string()
        } else if let Some((_, ret)) = pt.split_once("->").filter(|_| has_default) {
            // `by: T -> K` with `K: Ord`: a key that has no order.
            if bounded(ret.trim()) {
                args.push(format!("{pn}: x => bad"));
                hit = true;
            }
            continue;
        } else if has_default {
            continue;
        } else {
            return None;
        };
        hit |= value.contains("bad");
        if pn == "self" {
            recv = Some((value, mutable));
        } else {
            args.insert(0, value);
        }
    }
    if !hit {
        return None;
    }
    let args = args.join(", ");
    let call = match recv {
        Some((v, mutable)) => {
            let kw = if mutable { "var" } else { "let" };
            format!("{kw} xs = {v}\nprint(xs.{name}({args}))\n")
        }
        None => format!("print({name}({args}))\n"),
    };
    Some((name.to_string(), call))
}

#[test]
fn std_bounds_reject_calls_that_break_them() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("std_bounds");
    std::fs::create_dir_all(&dir).unwrap();
    let mut files: Vec<_> = std::fs::read_dir(root.join("std"))
        .expect("std/ exists")
        .map(|e| e.expect("readable entry").path())
        .filter(|p| p.extension().is_some_and(|x| x == "cp"))
        .collect();
    files.sort();
    let (mut generated, mut wrong) = (0, Vec::new());
    for file in &files {
        let module = file.file_stem().unwrap().to_string_lossy().to_string();
        let text = std::fs::read_to_string(file).unwrap();
        for sig in signatures(&text) {
            let Some((name, call)) = violating_call(&sig) else {
                continue;
            };
            generated += 1;
            let program = format!("use std.{module}.{{{name}}}\nlet bad = (x: Int) => x\n{call}");
            let path = dir.join(format!("{module}_{name}.cp"));
            std::fs::write(&path, &program).unwrap();
            let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
                .arg("check")
                .arg(&path)
                .current_dir(&root)
                .output()
                .expect("cpl runs");
            let stderr = String::from_utf8_lossy(&out.stderr);
            let at = stderr.lines().find(|l| l.trim_start().starts_with("-->"));
            let in_caller = at.is_some_and(|l| l.contains(&*path.to_string_lossy()));
            if out.status.success() || !in_caller {
                wrong.push(format!("std/{module}.cp `{name}`:\n{program}{stderr}"));
            }
        }
    }
    assert!(
        generated >= AT_LEAST,
        "only {generated} bound-breaking calls generated, at least {AT_LEAST}"
    );
    assert!(
        wrong.is_empty(),
        "calls that break a std bound must be rejected at the call:\n{}",
        wrong.join("\n")
    );
}
