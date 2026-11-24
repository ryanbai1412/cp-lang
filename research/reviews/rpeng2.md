# rpeng2 review

Score: 4/5
Dream: 4/5 (the friction's `## Dream`: the gap's ideal over the port)

Scored cold against the checklist and the port.

Cold review: rethought around one `Span = (lo:, hi:)` and one `join(op, a, b) -> Span`, so the
+ and * cases are a single function instead of two hand loops over lo/hi tables with 1e18
sentinels. Each chain's span is the min of lows and max of highs over its last-merge splits, a
comprehension. The ring is read typed as `[V](op: Char, x: -32768..=32767)`, chain lengths are
a `Len` domain, and the cut list is one comprehension.

## Checklist failures

- `for j in [V.nth((i.ord + left + 1) % n)]` binds a cyclic successor through a one-element list inside the comprehension, with ordinal arithmetic mod n.
- `span[i, Len(left)]` and `Len(len.ord - left - 1)` convert ordinals back into `Len` by hand.
- `span: [V, Len]Span = (lo: 0, hi: 0)` fills with a placeholder that every cell overwrites.
- `if op == 't'` dispatches on the raw operator character where a two-case enum read from its letter would name it.

## Gaps that need the language

- A cyclic successor as label arithmetic: research/frictions/rpeng2.md.
