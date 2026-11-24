//! Rust habits the checker recovers: `xs.max_by(f)` and its free twins.

use super::*;

/// `xs.max_by(f)`, the C++/Rust spelling: options are named arguments,
/// so it is `xs.max(by: f)`. Only for a base std declares with `by:`.
pub(crate) fn by_twin(src: &str, name: &str, span: Span) -> Option<(String, Fix)> {
    let base = by_base(name)?;
    let open = span.end as usize;
    let mut edits = vec![(span, base.to_string())];
    if src.get(open..open + 1) == Some("(") && src.get(open + 1..open + 2) != Some(")") {
        edits.push((Span::new(span.end + 1, span.end + 1), "by: ".into()));
    } else {
        return None;
    }
    Some((
        format!("write `.{base}(by: ..)`: options are named arguments, so there is one `{base}` and no `_by` twin"),
        Fix {
            title: habits::BY_TWIN.say(&[&base]),
            edits,
        },
    ))
}

/// The one std name a `_by` twin (`max_by`, `sort_by_key`) folds into.
fn by_base(name: &str) -> Option<&'static str> {
    let base = name.strip_suffix("_by").or(name.strip_suffix("_by_key"))?;
    Some(match base {
        "max" => "max",
        "min" => "min",
        "sort" => "sort",
        "sorted" => "sorted",
        "argmax" | "position_max" => "argmax",
        "argmin" | "position_min" => "argmin",
        "is_sorted" => "is_sorted",
        "runs" => "runs",
        "group" => "group",
        _ => return None,
    })
}

/// `min_by(xs, f)`, a `_by` twin called as a free function: `xs.min(by: f)`.
pub(crate) fn free_by_twin(src: &str, name: &str, span: Span) -> Option<(String, Fix)> {
    let base = by_base(name)?;
    let rest = src.get(span.end as usize..)?.strip_prefix('(')?;
    let mut depth = 0;
    let mut comma = None;
    for (i, c) in rest.char_indices() {
        match c {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' if depth == 0 => break,
            ')' | ']' | '}' => depth -= 1,
            ',' if depth == 0 => {
                comma = Some(i);
                break;
            }
            '"' | '\'' => return None,
            _ => {}
        }
    }
    let comma = comma?;
    let recv = rest[..comma].trim();
    let recv = recv.strip_prefix("mut ").unwrap_or(recv);
    if recv.is_empty() {
        return None;
    }
    let recv = if recv.contains(' ') || recv.contains("..") {
        format!("({recv})")
    } else {
        recv.to_string()
    };
    let args = &rest[comma + 1..];
    let end = span.end + 1 + (comma + 1 + args.len() - args.trim_start().len()) as u32;
    let shown = format!("{recv}.{base}(by: ..)");
    Some((
        format!("write `{shown}`: options are named arguments, so there is one `{base}` and no `_by` twin"),
        Fix {
            title: habits::WRITE.say(&[&shown]),
            edits: vec![(Span::new(span.start, end), format!("{recv}.{base}(by: "))],
        },
    ))
}
