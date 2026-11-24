# Friction Report: ccc18s4

## `@memo` on a sparse 1e9 key space is 1.5x the hand-tuned C++
- **Where:** `@memo fn trees(w: 1..=1e9)`.
- **What felt wrong:** the reached keys are only the `n / d` values (about 2 sqrt n); the accepted
  C++ caches small keys in a dense 1e7 array and the rest in `gp_hash_table`, about 0.13 s at
  `--max` against the port's 0.19 s. The port has no spelling for "memo over the quotients of n".
- **Goal it hurts:** performance.
- **Ideal code:** `@memo(array) fn trees(w: n.quotients)`: a domain of the `n / d` values, indexed
  densely (`w <= sqrt n` by `w`, else by `n / w`).
- **General fix:** a `Quotients(n)` domain in `std.math` next to `floor_quotients`, usable as a
  memo parameter type.
- **Severity:** medium

## Dream

```cp
let n: 1..=1e9 = read()
@memo(array)
fn trees(w: n.quotients) -> Int =
    if w == 1: 1 else: sum((b.hi - b.lo) * trees(b.q) for b in floor_quotients(w, from: 2))
print(trees(n))
```

## Measurements

- Tokens: port 74, original 606
- Max input (`gen.py --max`): port 0.20 s, 19484 KB, original 0.13 s, 81792 KB
- Emitted C++: 18602 bytes
