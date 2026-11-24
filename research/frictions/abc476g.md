# abc476g friction

There is no `dream.cp`; the ideal is below. `orig.cpp` is not judge-accepted: it was written
for the corpus and stress-checked against the port and its brute.

## Dream
The port, with the out-of-range reads of `f` spelled as an optional read:

```cp
f[k, a, b] = max(
    (f.get(k - 1, min(a, k - 1), c) ?? 0) + (if c == 0: 0 else: f.get(k - 1, c - 1, b - 1) ?? 0)
    for c in b..=a
)!
```

## Out-of-table reads guarded by hand
- **Where:** `port.cp`, the table build:
  `(if c > k - 1: 0 else: f[k - 1, min(a, k - 1), c]) + (if c == 0 or b - 1 > k - 1: 0 else: f[k - 1, min(c - 1, k - 1), max(0, b - 1)])`
- **What felt wrong:** "no such subsequence counts 0" is written as two hand range guards plus
  `min`/`max` clamps that keep the indices in the table's domain.
- **Goal it hurts:** express, hard-to-get-wrong (a wrong clamp reads a real cell silently).
- **Ideal code:** the dream above, where a read outside the table's `b <= a <= k` triangle is
  `None`.
- **Run:** `var f: [0..=2, 0..=2]Int = 0` then `print(f.get(5, 0) ?? 0)` under `cpl run`
  (09954f019): `error[E0303]: 'get' takes 'i: Int', as in 'a.get(1)'`.
- **General fix:** a checked read on a multi-index table (`t.get(i, j, k) -> ?T`), like the
  one-index `get` that exists today. Every DP that treats an outside cell as 0 would use it.
- **Severity:** low.
- **Owner:** Content (stdlib arrays).

## Measurements
- Tokens: port 472, original 529.
- `--max` (best of 10): port 0.254 s / 4 MB, `orig.cpp` 0.396 s / 4 MB. Same output.
- Emitted C++: 16,807 bytes.
