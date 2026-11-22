# ucup5s02f

Score: pending cold rescore (wrote dream + rewrote port with header deriving `covers` and 6/3/2 growth from `6 / gcd(6, v)`, bounded inputs, typed `[0..n0]`/`[0..n1]`/`[0..q]` reads with zero turbofish, `?Int` return, and hoisted `k6`/`t6`/`shift` — 0 lints; emitted C++ 8,122 -> 10,650 B (+2,528 B, < 48 KB) from typed table reads + `?Int` optional + input range checks; shared read helpers will trim it)
Dream: pending cold rescore (new dream matching active port)

Baseline review, cold, on main 8b49fa9d. A line-by-line translation with no header. The search over `k` (lines 12-18) is the whole idea, and nothing says why `k` grows by a factor of 6, 3 or 2. No dream exists, so the score is capped at 3.

## Checklist failures

- No header: neither `covers` (lines 1-9) nor the `6 / 3 / 2` growth rule (lines 15-17) is stated or derived.
- `I128` appears 26 times (lines 1-25). `n0`, `n1`, `q` and the segment ends are all unbounded `Int` (lines 21-24), so no width comes from a type.
- `-1` is a sentinel return (lines 14, 19) where `?Int` says "impossible".
- `read::<(Int, Int)>()` in a comprehension (lines 22-23) is turbofish drift; a typed `[0..n0](Int, Int) = read()` reads the table.

## Rewrites owed with today's language

- Write a header that derives `covers` and why the factor is 6, 3 or 2.
- Bound the inputs from the statement so `I128` follows from the types, not hand casts.
- Read `p0`/`p1` as typed tables. Return `?Int` and print `?? -1` at the call.

## Gaps that need the language

- None found; the port has no friction report.
