//! `@law` at the top level of a file: a property `cpl test` checks.
//!
//! `@law comm(a: Int, b: Int) = add(a, b) == add(b, a)` is run as a test
//! named `law comm`: the condition is asserted for every combination of
//! sample values of its arguments. A broken law fails with the values
//! that break it.

use cpl_syntax::ast::{StmtKind, TypeKind};

/// The values a law's argument is tried with, as source text.
fn samples(ty: Option<&str>) -> Option<String> {
    Some(match ty {
        None | Some("Int" | "I64") => {
            "[0, 1, -1, 2, -2, 3, 5, -7, 10, 64, 1000, 1000000007, -1000000007]".to_string()
        }
        Some("Bool") => "[false, true]".to_string(),
        Some("F64" | "Float") => "[0.0, 1.0, -1.0, 0.5, 2.0, -3.25, 1000.0]".to_string(),
        Some("Char") => "['a', 'b', 'z', 'A', '0', ' ']".to_string(),
        Some("Str") => "[\"\", \"a\", \"b\", \"ab\", \"ba\", \"abc\"]".to_string(),
        // A domain or an enum is walked whole.
        Some(name) if name.starts_with(|c: char| c.is_ascii_uppercase()) => name.to_string(),
        Some(_) => return None,
    })
}

/// The name of the test a law runs as.
pub fn test_name(law: &str) -> String {
    format!("law_{law}")
}

/// `src` with one `@test` after it for each top-level `@law`, and the
/// names of those tests. `None` when the file has no such law.
pub fn with_law_tests(src: &str) -> Option<(String, Vec<String>)> {
    let parsed = cpl_syntax::parse_file(src);
    let mut out = String::new();
    let mut names = Vec::new();
    for (offset, item) in &parsed.items {
        for stmt in &item.stmts {
            let StmtKind::Law { name, params, body } = &stmt.kind else {
                continue;
            };
            let text = |start: u32, end: u32| &src[(offset + start) as usize..(offset + end) as usize];
            let mut loops = String::new();
            let mut shown = Vec::new();
            let mut depth = 1;
            for param in params {
                let ty = match param.ty.as_ref().map(|ty| (&ty.kind, ty.span)) {
                    None => None,
                    Some((TypeKind::Named { args, .. }, span)) if args.is_empty() => {
                        Some(text(span.start, span.end))
                    }
                    // A range type is walked when it is small: `0..=9`.
                    Some((TypeKind::Range { .. }, span)) => Some(text(span.start, span.end)),
                    Some((_, span)) => Some(text(span.start, span.end)),
                };
                let values = match param.ty.as_ref().map(|ty| &ty.kind) {
                    Some(TypeKind::Range { .. }) => ty.map(str::to_string),
                    Some(TypeKind::Named { .. }) | None => samples(ty),
                    Some(_) => None,
                };
                let values = values.unwrap_or_else(|| {
                    format!("__law_needs_samples_for_{}", param.name.text)
                });
                loops += &format!("{}for {} in {values}:\n", "    ".repeat(depth), param.name.text);
                shown.push(format!("{} = {{{}}}", param.name.text, param.name.text));
                depth += 1;
            }
            let cond = text(body.span.start, body.span.end);
            let test = test_name(&name.text);
            out += &format!(
                "\n@test\nfn {test}():\n{loops}{}assert ({cond}), \"law `{}` does not hold for {}\"\n",
                "    ".repeat(depth),
                name.text,
                shown.join(", "),
            );
            names.push(test);
        }
    }
    if names.is_empty() {
        return None;
    }
    let mut full = src.to_string();
    if !full.ends_with('\n') {
        full.push('\n');
    }
    full += &out;
    Some((full, names))
}
