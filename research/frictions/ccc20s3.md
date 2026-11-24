# Friction Report: ccc20s3

## Window permutation test is hand-rolled
- **Where:** `fn shift(c, by)` keeping `surplus` and the `off` count of nonzero letters.
- **What felt wrong:** "the window's letter multiset equals the needle's" is a standard sliding
  multiset comparison; the port maintains the counter of mismatches by hand.
- **Goal it hurts:** brevity; common mistakes hard to write.
- **Ideal code:** `for (i, w) in hay.windows(k).with_counts(over: Letter).enumerate() if w == want:`.
- **General fix:** a sliding `windows(k).counts(over: D)` that tracks equality with a target in
  O(1) per step.
- **Severity:** medium

## The accepted C++ needs more than the 8 MB stack
- **Where:** stress against `corpus/submissions/4323643.cpp`.
- **What felt wrong:** its three hashers hold 9.6 MB of arrays in `solve()`'s frame, so it
  crashes under the default 8 MB stack; it agrees on all 30 cases under `ulimit -s unlimited`.
- **Goal it hurts:** gate, then compare.
- **Ideal code:** n/a (the C++ original's issue).
- **General fix:** none in the port.
- **Severity:** low

## Dream

```cp
domain Letter = 'a'..='z'
let needle: Str = read()
let hay: Str = read()
let want = needle.counts(over: Letter)
let h = RollingHash(hay)
print(Set(h.get(w.span) for w in hay.windows(needle.len).with_counts(over: Letter) if w.counts == want).len)
```

## Measurements

- Tokens: port 213, original 893
- Max input (`gen.py --max`): port 0.00 s, 9728 KB, original 0.06 s, 13312 KB
- Emitted C++: 21035 bytes
