# atcoder_dp_o

Score: 4/5
Dream: 4/5

Cold review of `port.cp` against `dream.cp` (the same code), without the C++. `ways: [Taken]Z`
over `Subset<Woman>`, `Man(t.len - 1)` for the next man, and `sum(... for w in t if likes[m, w]
== 1)` are the editorial's recurrence.

## Checklist failures
- idiomatic-code/R2 (minor): `if t != Taken.empty:` wraps the whole loop body; `for t in Taken
  if t != Taken.empty:` says it once.
- `const P = 1000000007` lacks digit separators (style).

## Owed rewrites
- The filter on the `for`.

## Language gaps
None.

## Dream notes
The dream is the same code, so it has the same nit.
