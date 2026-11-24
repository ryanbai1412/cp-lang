# Friction Report: ccc05s5

## Counting the keys above a value
- **Where:** `seen.len - seen.rank(s + 1)`.
- **What felt wrong:** "how many earlier scores beat this one" is a suffix count; the port
  spells it as the complement of a rank at `s + 1`, which only works for integer keys.
- **Goal it hurts:** brevity; common mistakes hard to write.
- **Ideal code:** `seen.count_gt(s)` (as `MergeSortTree` has).
- **General fix:** `count_lt/le/gt/ge` on `SortedBag`, matching `MergeSortTree`.
- **Severity:** low
- **Run:** `var seen = SortedBag<Int>(); seen.add(3); print(seen.count_gt(2))` under `cpl run`
  (c8a46b201): `` error[E0203]: SortedBag<Int> has no field or method `count_gt` ``.

## Dream

```cp
let t: 1..=1e5 = read()
let scores: [t]0..=1e9 = read()
var seen = SortedBag<Int>()
print(fixed(sum(1 + seen.count_gt(s) after seen.add(s) for s in scores), t, 2))
```

## 1.75x slower than the accepted C++ at --max
- **Where:** the `SortedBag` loop, t = 1e5.
- **What felt wrong:** port 0.07 s against the accepted C++'s 0.04 s (the measurement below),
  with a 200 ms limit, over the 1.1x line (idiomatic-code/R10). Not profiled.
- **Goal it hurts:** performance.
- **Ideal code:** the port as written.
- **General fix:** profile `SortedBag.add`/`rank` against the C++'s structure and fix the std
  ordered multiset, so every rank-query port gains.
- **Severity:** medium (the limit is 200 ms).
- **Owner:** perf. Repro: `gen.py --max | port` against `orig.cpp`.

## Measurements

- Tokens: port 79, original 413
- Max input (`gen.py --max`): port 0.07 s, 9500 KB, original 0.04 s, 10368 KB
- Emitted C++: 16622 bytes
