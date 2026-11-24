# cco21p5 friction

## The suffix-min search over cuts doesn't type

- **Where:** `spec.cp`, the `start` pointer: "the first cut after which every low is at least v".
- **What felt wrong:** the natural form is a suffix-min scan and a binary search over its cuts,
  `let suf = low.rev.scan(min)` then `V.cuts.first_true(c => suf[c] >= v)!`; neither types
  today, so spec.cp walks a cut pointer left by hand.
- **Goal it hurts:** express, coherence (R07.1.6 and R04.4.19 both promise the form).
- **Ideal code:** `dp.fold(V.cuts.first_true(c => low.rev.scan(min)[c] >= v)!..)`, with the scan
  hoisted.
- **Repro:** `domain V = 1..=4; let suf = ([V] v => Int(v)).rev.scan(+); print(suf[V(3).r])` is
  E0302 "expected `D.cuts`, found `V.cuts`"; `let c: Cut<V> = V.cuts.first_true(c => Int(c) >= 2)!`
  is E0302 "expected `V.cuts`, found `V`"; `suffix_fold(min, low)` on `low: [V]V` is E0501
  (`.MAX` of a domain as a value).
- **General fix:** bind `D` through `Rev<[D]T>` in `scan`; type `V.cuts` as the cuts domain so its
  range methods return cuts (compiler worklog, 2026-11-23).
- **Severity:** low (the pointer is O(n) and reads like the editorial).

## A search over a range of cuts types its closure as positions

- **Where:** `port.cp`, `next`: the first cut from `i.r` on with no earlier neighbour before `i`.
- **What felt wrong:** `(i.r..=end).first_true(c => later[c].is_none_or(e => e >= i))` is E0301
  "indexed by `Town.cuts`, but the index is a `Town`": the range is `Range<Town.cuts>` but the
  closure parameter is typed as a town. The port searches the cut ordinals as `Int` and converts
  back with `Town.cuts(k)`.
- **Goal it hurts:** express (a round trip through `Int` for a search that is about cuts).
- **Ideal code:** `let next = (i.r..=end).first_true(c => later[c].is_none_or(e => e >= i))!`.
- **Repro:** `domain T = 1..=5; let later: [T.cuts]Int = [T.cuts] c => Int(c);
  let r: Range<T.cuts> = T(2).r..=T.cuts.last!; print(Int(r.first_true(c => later[c] >= 3)!))`
  is E0301; sent to Core.
- **Severity:** medium.

## `min` as a function value loses the None-is-identity rule

- **Where:** `port.cp`, the SegTree monoid over `?Int` minimums.
- **What felt wrong:** `Monoid<?Int>(None, min)` checks, but its `op(None, Some(3))` is `None`,
  while a direct `min(None, Some(3))` is `Some(3)`; the tree then answers `None` for every fold.
  The port writes the lambda `(x, y) => min(x, y)`, which keeps the rule.
- **Goal it hurts:** correct (a silent wrong answer), coherence (two meanings for one name).
- **Ideal code:** `over: Monoid<?Int>(None, min)`.
- **Repro:** `let a: ?Int = None; let b: ?Int = Some(3);
  print(min(a, b) ?? -1, Monoid<?Int>(None, min).op(a, b) ?? -1)` prints `3 -1`; sent to Core.
- **Severity:** high (wrong answer with no diagnostic).

## An open range from a cut reads as the pieces after it

- **Where:** `port.cp`, `best.fold(next..=end)` and `best.apply(v.r..=end, 1)`.
- **What felt wrong:** on a SegTree indexed by `Town.cuts`, `best.fold(next..)` from the last cut
  folded nothing: the open form is read as the towns after the cut, not the cuts from it on. The
  port spells the closed range with `end = Town.cuts.last!`.
- **Goal it hurts:** express, correct (the open form checks and silently drops the last cut).
- **Ideal code:** `best.fold(next..)`.
- **Severity:** medium.

## Measurements

Largest generated input (n = 80,691, m = 151,508): accepted C++ 0.04 s, interpreter 3.1 s; both
print 59233.

Current port: 30 generated cases agree with the old port and 30 with the accepted C++; the
`--max` input (n = m = 2e5) matches the accepted C++, port 0.11 s against 0.07 s.
