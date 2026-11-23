//! The CP Lang fix a trap's diagnostic suggests, from its message.

/// How to fix the commonest traps, in CP Lang terms.
pub(crate) fn help(message: &str) -> Option<String> {
    let m = message;
    let m = m.strip_prefix("reading the number of tests: ").unwrap_or(m);
    if m.starts_with("the input ended") {
        return Some(
            "the program reads more values than the input has: check the loop bounds \
                 (`0..n` reads n values, `0..=n` reads n + 1) and the order of the reads"
                .into(),
        );
    }
    if m.starts_with("`!` on `None`") || m.starts_with("there is no value here") {
        return Some(
            "`x!` needs a value: to handle `None`, write `if let Some(v) = x:`, \
                 in a loop `while let Some(v) = x and v > k:`, or `x ?? fallback`"
                .into(),
        );
    }
    let (idx, rest) = m.strip_prefix("index ")?.split_once(" is outside ")?;
    let len: i64 = rest.rsplit(' ').next()?.parse().ok()?;
    if idx.starts_with('-') {
        return rest.contains("a list").then(|| {
            "CP Lang has no negative indexes: the last item is `xs.last!`, or `xs[xs.len - 1]`"
                .into()
        });
    }
    Some(if len == 0 {
        "it is empty: check `xs.len > 0` first, or use `xs.first` (a `?T`)".into()
    } else {
        format!(
            "the indexes are 0 to {}: a loop over `0..=n` reaches `n`, `0..n` stops before it",
            len - 1
        )
    })
}
