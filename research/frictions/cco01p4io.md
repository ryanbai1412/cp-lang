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
