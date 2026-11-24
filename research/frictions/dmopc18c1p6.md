# Friction report: dmopc18c1p6

The DP runs over pairs of cuts, so an empty or one-row block needs no case and the split is a
cut, not "k from l - 1 to r". Two gaps remain against the dream.

## A cut can't step by k

- **Where:** `if let b = Row.cuts.at(Int(a) + span):`
- **What felt wrong:** positions step with `v.next(k)`; cuts only have `c.next`, and
  `c.next(span)` resolves to something else entirely, so the port drops to `Int` and back.
- **Goal it hurts:** express (one rule for positions and cuts)
- **Ideal code:** `if let b = a.next(span):`
- **Repro (fresh build):**
  ```
  domain P = 0..4
  let c = P.cuts.first!
  print(c.next(2)!)
  ```
  `error[E0302]: expected [Int]List<Int>, found P.cuts`
- **General fix:** check/stdlib: `c.next(k)` / `c.prev(k)` on cuts, as on positions.
- **Severity:** medium (the error message points nowhere near cuts)

## A table of `Prefix` is about 30% slower than a flat table

- **Where:** `let ones_before: [Col, Row.cuts]Int = [Col, Row.cuts] (c, k) => column_ones[c].at(k)`
- **What felt wrong:** the dream reads `column_ones[c].fold(a..s)` in the inner loop. That
  costs 0.46 s on the max case against 0.35 s for the flat `[Col, Row.cuts]` copy (accepted
  C++ 0.37 s): each `[Col]` element is its own heap array, and `fold` re-tests for an empty
  range.
- **Goal it hurts:** perf (the obvious spelling is the slow one)
- **General fix:** codegen: store `[D]Prefix<E, T>` as one flat `[D, E.cuts]` block and inline
  `fold` on a proved non-empty range; or stdlib: a column-wise `Prefix` over `[R, C]` with
  `fold(row, cuts)`.
- **Severity:** medium

## Measurements

gen.py --max (n = 1000, m = 50), three runs: port 0.37 / 0.45 / 0.40 s, accepted C++
0.36 / 0.43 / 0.35 s (needs a big stack: its DP is a local array).

## Dream

```cp
let n: 1..=1000 = read()
let m: 1..=50 = read()
domain Row = 0..n
domain Col = 0..m
let bits: [Row, Col]Char = read()
let ones = [Col] c => Prefix([Row] r => Int(bits[r, c] == '1'))
fn flips(c: Col, a: Row.cuts, s: Row.cuts, b: Row.cuts) -> Int =
    ones[c].fold(a..s) + (Int(b) - Int(s)) - ones[c].fold(s..b)
var fewest: [Row.cuts, Row.cuts]Int = 0
for c in Col.rev:
    var here: [Row.cuts, Row.cuts]Int = 0
    var turn: [Row.cuts, Row.cuts]Row.cuts = Row.cuts.first!
    for span in (2..=n).rev:
        for a in Row.cuts:
            if let b = a.next(span):
                let lo = if let wider = a.prev: max(a, turn[wider, b]) else: a
                let hi = if let wider = b.next: min(b, turn[a, wider]) else: b
                (here[a, b], turn[a, b]) = min((flips(c, a, s, b) + fewest[a, s] + fewest[s, b], s) for s in lo..=hi)!
    fewest = here
print(fewest[Row.cuts.first!, Row.cuts.last!])
```
