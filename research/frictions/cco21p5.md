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

## Measurements

Largest generated input (n = 80,691, m = 151,508): accepted C++ 0.04 s, interpreter 3.1 s; both
print 59233.
