//! Reading `sqrt` out of a call's source text.

/// `n` from the text `Int(sqrt(n))`, `Int(sqrt(F64(n)))` or `Int(n.sqrt)`,
/// when the text is exactly one of those.
pub(super) fn sqrt_operand(text: &str) -> Option<String> {
    let inner = text.strip_prefix("Int(")?.strip_suffix(')')?.trim();
    let arg = match inner.strip_prefix("sqrt(") {
        Some(rest) => rest.strip_suffix(')')?,
        None => inner.strip_suffix(".sqrt")?,
    };
    let arg = arg
        .strip_prefix("F64(")
        .and_then(|a| a.strip_suffix(')'))
        .unwrap_or(arg)
        .trim();
    let mut depth = 0i32;
    for c in arg.chars() {
        depth += match c {
            '(' | '[' => 1,
            ')' | ']' => -1,
            _ => 0,
        };
        if depth < 0 {
            return None;
        }
    }
    (depth == 0 && !arg.is_empty()).then(|| arg.to_string())
}
