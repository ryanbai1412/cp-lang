# Friction report: noip21p3

There is no dream.cp; the ideal is below. The port sorts the gaps and memoizes `rest(i, s)`. Each gap goes on the right (a new value as tall as all gaps so far) or the left (lifting the i + 1 earlier values), and `@memo` keeps only the reached states.

## Dream

```cp
let a = read<[n]1..=600>()
let gaps = a.diffs().sorted()
let height = prefix_fold(+, gaps)
@memo
fn rest(i: 0..n, s: 0..=top) -> Int: ...
```

## Adjacent differences are an index comprehension
- **Where:** line 19: `sorted([a[i + 1] - a[i] for i in 0..n - 1])`.
- **What felt wrong:** "the gaps of a" is written with an index, its successor and the `n - 1` end.
- **Goal it hurts:** express, hard-to-get-wrong (an off-by-one in the end is easy).
- **Ideal code:** `a.diffs()` (or `a.windows(2).map(w => w[1] - w[0])`).
- **General fix:** a std `diffs()` on lists and arrays, the inverse of `prefix_fold(+)`. Owner: stdlib.
- **Severity:** low.

## Measurements

- **Tokens:** port 192, accepted C++ 735 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.06 s / 11 MB, accepted C++ 0.08 s / 22 MB, with the same output.
- **Emitted C++:** 28941 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
