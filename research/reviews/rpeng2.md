# rpeng2 review

Score: pending
Dream: pending

Cold review: rethought around one `Span = (lo:, hi:)` and one `join(op, a, b) -> Span`, so the
+ and * cases are a single function instead of two hand loops over lo/hi tables with 1e18
sentinels. Each chain's span is the min of lows and max of highs over its last-merge splits, a
comprehension. The ring is read typed as `[V](op: Char, x: -32768..=32767)`, chain lengths are
a `Len` domain, and the cut list is one comprehension.
