# apio07p2

Ported from `corpus/submissions/5532752.cpp`. No `dream.cp`; the dream is below.

## Dream
```cp
let n: 2..=1e5 = read()
let k: 1..=n / 2 = read()
let at: [n]0..=1e9 = read()
let gaps = [b - a for (a, b) in at.adjacent]

// the least cost of cables priced at lam each, fewest cables on ties
fn priced(lam: Int) -> (val: Int, count: Int):
    var before = (val: 0, count: 0)  // best over the gaps up to the one before last
    var best = (val: 0, count: 0)
    for g in gaps:
        let take = (val: before.val + g + lam, count: before.count + 1)
        before = best
        best = min(best, take)
    best

print(aliens(k, priced, lambdas: -2_000_000_001..=0))
```

## Positional state where the result type is named
- **Where:** `port.cp` `fn priced`: `var before = (0, 0)`, `before.0 + g + lam`, `before.1 + 1`,
  then `(val: best.0, count: best.1)`.
- **What felt wrong:** the function's result is named `(val:, count:)`, but its two running
  values are positional pairs, converted at the end. The reader maps `.0` to cost and `.1` to
  cables by hand.
- **Goal it hurts:** one spelling per concept (idiomatic-code/R8) and readability
  (idiomatic-code/R2).
- **Ideal code:** the dream above: the state is the named record, and `min` on it still orders
  by `val`, then `count`.
- **General fix:** a port rewrite; the language already has it. Named records compare
  field by field in declaration order: `min((val: 3, count: 1), (val: 3, count: 0))` prints
  `3 0` under `cpl run`.
- **Severity:** low.
- **Owner:** idioms (this port); lands with the next port-edit commit (the `copies` slice is red
  on main).

## Measurements
- Tokens: port 159, original 1,062.
- `--max` (best of 3): port 7.6 ms / 4.6 MB, original 264 ms / 5.9 MB (0.03x). Same output.
  The original binary-searches λ over a wider range with a heavier DP.
- Emitted C++: 14,554 bytes.
