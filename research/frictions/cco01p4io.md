# Friction Report: cco01p4io

The v0.2 port memoized a recursive `dp` in a `[0..105]Int` seeded with the sentinel `-1`, took
the minimum from a `1_000_000_000` sentinel, returned 0 for two candidate levels (one drop too
few), and searched recursively with an index loop for the second sample. Now `drops` is a
bottom-up table over `domain Span = 0..n`, each entry one `min` over `(cost, i)` pairs, and the
search is a loop over the candidate range whose second-sample pass is
`(lo..at).find(breaks) ?? at`. The package gains gen.py with the tight drop budget, an
interactor that enforces the budget and the two samples, and tests.toml. The accepted C++
passes the same interactor.

## `ask` replies cannot be declared in `// Bounds:`
- **Where:** `n` and `d` come from `ask("GetN")` and `ask("GetD")`; the header gate reports them
  as unread, so their statement bounds live only in the types.

## A DP table cannot be defined from itself
- **Where:** `var drops: [Span]Int` and `var up: [Span]Int`, filled by a `for s in Span if s.idx > 0` loop.
- **What felt wrong:** the dream's `let plan = [Span] s => ... plan[s - i - 1].cost ...` is the
  recurrence as written. `cpl run` of
  `let plan: [Span](cost: Int, up: Int) = [Span] s => ... plan[Span(s.ord - i - 1)].cost ...`
  fails with `error[E0205]: plan is used in its own definition`.
- **Goal it hurts:** quick to write; each fact once (two arrays kept in step).
- **Ideal code:** `let plan = [Span] s => min((1 + max(i, plan[s - i - 1].cost), i) for i in 0..s)`.
- **General fix:** a self-referential array builder (`Span.build((s, plan) => ...)` reading only
  earlier entries), or allow the self-reference when every read is at a smaller index.
- **Severity:** low
- **Owner:** stdlib.

## Dream
```cp
let (n, d): (1..=100, 1..=100) = (ask("GetN"), ask("GetD"))
let plan = [Span] s => min((1 + max(i, plan[s - i - 1].cost), i) for i in 0..s)  // self-referential table
var (lo, hi) = (1, n)
while lo < hi:
    let at = lo + plan[hi - lo].up
    if ask("Drop", at): lo = (lo..at).find(j => ask("Drop", j)) ?? at; hi = lo
    else: lo = at + 1
print("Decide", lo)
```
