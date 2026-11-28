//! `@law` at the top level of a file: a property `cpl test` checks.
//!
//! `@law comm(a: Int, b: Int) = add(a, b) == add(b, a)` is run as a test
//! named `law comm`: the condition is asserted for every combination of
//! sample values of its arguments. A broken law fails with the values
//! that break it.

use cpl_diag::Span;
use cpl_facade::today::Ty;
use cpl_syntax::ast::StmtKind;

const INTS: [i128; 13] = [
    0,
    1,
    -1,
    2,
    -2,
    3,
    5,
    -7,
    10,
    64,
    1000,
    1000000007,
    -1000000007,
];

/// A range this short is walked whole: `0..=9`.
const WALK_MAX: i128 = 64;

/// The values a law's argument is tried with, as source text, by the type
/// the checker resolved its written type to (`None`: no type written).
fn samples(ty: Option<&Ty>) -> Option<String> {
    let list = |xs: &[&str]| format!("[{}]", xs.join(", "));
    Some(match ty {
        None | Some(Ty::Int | Ty::I128) => list(
            &INTS
                .map(|x| x.to_string())
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
        ),
        Some(Ty::Range { lo, hi, .. }) => match (lo, hi) {
            (Some(lo), Some(hi)) if hi - lo <= WALK_MAX => format!("{lo}..={hi}"),
            _ => {
                let (lo, hi) = (lo.unwrap_or(i128::MIN), hi.unwrap_or(i128::MAX));
                let ends = [lo, lo.saturating_add(1), hi.saturating_sub(1), hi];
                let mut xs: Vec<i128> = (INTS.iter().chain(&ends).copied())
                    .filter(|x| (lo..=hi).contains(x) && x.unsigned_abs() <= i64::MAX as u128)
                    .collect();
                xs.sort_unstable();
                xs.dedup();
                let xs: Vec<String> = xs.iter().map(i128::to_string).collect();
                list(&xs.iter().map(String::as_str).collect::<Vec<_>>())
            }
        },
        Some(Ty::Bool) => list(&["false", "true"]),
        Some(Ty::Float) => list(&["0.0", "1.0", "-1.0", "0.5", "2.0", "-3.25", "1000.0"]),
        Some(Ty::Char) => list(&["'a'", "'b'", "'z'", "'A'", "'0'", "' '"]),
        Some(Ty::Str) => list(&["\"\"", "\"a\"", "\"b\"", "\"ab\"", "\"ba\"", "\"abc\""]),
        // A position of a domain, or an enum, is walked whole.
        Some(Ty::Pos(domain)) => domain.clone(),
        Some(Ty::Named { name, args }) if args.is_empty() => name.clone(),
        Some(_) => return None,
    })
}

/// The name of the test a law runs as.
pub fn test_name(law: &str) -> String {
    format!("law_{law}")
}

/// `src` with one `@test` after it for each top-level `@law`, and the
/// names of those tests. `None` when the file has no such law. Each
/// argument's samples follow the type the checker resolved for it, so
/// `x: Small` with `type Small = 1..=9` is walked as `1..=9`.
pub fn with_law_tests(
    db: &mut cpl_facade::Session,
    path: &str,
    src: &str,
) -> Option<(String, Vec<String>)> {
    let parsed = cpl_syntax::parse_file(src);
    if !(parsed.items.iter()).any(|(_, item)| {
        item.stmts
            .iter()
            .any(|s| matches!(s.kind, StmtKind::Law { .. }))
    }) {
        return None;
    }
    *db = db.with_file(path, src);
    let facts = db.facts(path);
    let mut out = String::new();
    let mut names = Vec::new();
    for (offset, item) in &parsed.items {
        for stmt in &item.stmts {
            let StmtKind::Law { name, params, body } = &stmt.kind else {
                continue;
            };
            let text =
                |start: u32, end: u32| &src[(offset + start) as usize..(offset + end) as usize];
            let mut loops = String::new();
            let mut shown = Vec::new();
            let mut depth = 1;
            for param in params {
                let written = param.ty.as_ref().map(|ty| {
                    let at = Span {
                        start: offset + ty.span.start,
                        end: offset + ty.span.end,
                    };
                    facts
                        .as_ref()
                        .and_then(|f| f.written_type(at))
                        .unwrap_or(&Ty::Unknown)
                });
                let values = samples(written);
                let values = values
                    .unwrap_or_else(|| format!("__law_needs_samples_for_{}", param.name.text));
                loops += &format!(
                    "{}for {} in {values}:\n",
                    "    ".repeat(depth),
                    param.name.text
                );
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
