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
    if m.starts_with("a shift by ") {
        return Some(
            "`a << k` and `a >> k` need `k` from 0 to below the width (64, or 128 for `I128`): \
                 for a large power of two, use `**` on a `Mod<P>` value"
                .into(),
        );
    }
    if m.starts_with("integer overflow:") {
        return Some(if m.contains("(64 bits)") {
            "if the answer is asked mod P, keep it in `Mod<P>` values; otherwise widen this value to `I128`"
        } else {
            "if the answer is asked mod P, keep it in `Mod<P>` values, which never overflow"
        }
        .into());
    }
    if m.starts_with("division by zero:") {
        return Some(
            "the divisor can be 0 here: guard it with `if b != 0:`, \
                 or give it a type that leaves 0 out, as in `1..=n`"
                .into(),
        );
    }
    if m.starts_with("expected a number, found") {
        return Some(
            "the reads don't line up with the input: check their order and count against the input format"
                .into(),
        );
    }
    if m.starts_with("the stack ran out") {
        return Some(
            "each call must get closer to a case that returns without calling again: check the base case"
                .into(),
        );
    }
    if let Some((_, range)) = m
        .split_once(" is not in `")
        .and_then(|(_, r)| r.split_once(", which is "))
    {
        if m.contains("(input line") {
            return Some("the input breaks a domain you declared: check it against the statement's constraints".into());
        }
        let range = range.split(" (").next().unwrap_or(range);
        return Some(format!(
            "the value is outside the domain: check it with `if x in {range}:` first, or give the variable the domain's range"
        ));
    }
    if m.starts_with("assertion failed") {
        return Some(
            "the condition was false for the values below: fix the code that computed them, or the assertion if it is wrong"
                .into(),
        );
    }
    if m.starts_with("`todo` reached") {
        return Some("this input reaches an unwritten branch: replace `todo` with its code".into());
    }
    if let Some((_, range)) = m.split_once(" is outside `") {
        return Some(if range.contains("(input line") {
            "the input breaks a range you declared: check the type against the statement's constraints"
        } else {
            "the value leaves the range its variable was declared with: widen the type, or check the value first"
        }
        .into());
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
