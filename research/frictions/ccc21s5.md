# Friction Report: ccc21s5

## Sixteen difference arrays where one keyed structure would do
- **Where:** `fn covered(z)` builds a `Diff<I, Int>` per value z.
- **What felt wrong:** the idea is "which z cover i"; the port makes 16 passes over the
  requirements, one per z, instead of one `Diff` over a 16-bit mask with an OR-fold.
- **Goal it hurts:** brevity.
- **Ideal code:** `let a = Diff<I, Lcm>.from(reqs, range: q => q.l..=q.r, value: q => q.z).finish()`
  with `Lcm` the monoid; `Diff` today needs `Add + Sub`, so a non-invertible fold over ranges
  has no offline spelling.
- **General fix:** an offline range-assign-by-monoid (sweep with counts per value, or a sparse
  table built in reverse) in `std.prefix`.
- **Severity:** low

## Dream

```cp
let (n, m): (1..=150_000, 1..=150_000) = read()
domain I = 1..=n
let reqs: [m](l: I, r: I, z: 1..=16) = read()
let a = range_fold(lcm, reqs.map(q => (q.l..=q.r, q.z)), over: I, init: 1)
let gcds = SparseTable(gcd, a)
print(if reqs.all(q => gcds.fold(q.l..=q.r) == q.z): a else: "Impossible")
```

## Measurements

- Tokens: port 227, original 790
- Max input (`gen.py --max`): port 0.04 s, 11976 KB, original 0.06 s, 39680 KB
- Emitted C++: 21495 bytes
