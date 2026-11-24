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

/// The one std name a `_by` twin (`max_by`, `sort_by_key`) folds into: a std fn that takes
/// `by:`, or the std name itertools' `position_max` stands for.
fn by_base(name: &str) -> Option<&str> {
    let base = name.strip_suffix("_by").or(name.strip_suffix("_by_key"))?;
    let base = match base {
        "position_max" => "argmax",
        "position_min" => "argmin",
        base => base,
    };
    crate::prelude::std_takes_by(base).then_some(base)
}

/// `min_by(xs, f)`, a `_by` twin called as a free function: `xs.min(by: f)`.
pub(crate) fn free_by_twin(src: &str, name: &str, span: Span) -> Option<(String, Fix)> {
    let base = by_base(name)?;
    let rest = src.get(span.end as usize..)?.strip_prefix('(')?;
    let Some((comma, false)) = first_arg(rest) else {
        return None;
    };
    let recv = rest[..comma].trim();
    if recv.contains('"') || recv.contains('\'') {
        return None;
    }
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
