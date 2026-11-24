# Friction Report: ccc21s3

## Weighted median is spelled as a scan and a find
- **Where:** `reached = ends.map(e => e.w).collect().scan(+)` then `find(i => reached.prefix(i + 1) >= total)`.
- **What felt wrong:** the idea is "the weighted median of the ends"; the port builds prefix sums
  and searches them by index.
- **Goal it hurts:** brevity.
- **Ideal code:** `ends.weighted_median(by: e => e.w).at`.
- **General fix:** `weighted_median(by:)` on sequences in `std.seq` (it is the argmin of every
  sum of weighted absolute deviations, a recurring CP step).
- **Severity:** low

## Dream

```cp
let n: 1..=2e5 = read()
let friends: [n](p: 0..=1e9, w: 1..=1000, d: 0..=1e9) = read()
let c = [(at: f.p + s * f.d, w: f.w) for f in friends for s in [-1, 1]].weighted_median(by: e => e.w).at
print(sum(f.w * max(0, abs(f.p - c) - f.d) for f in friends))
```

## Measurements

- Tokens: port 201, original 460
- Max input (`gen.py --max`): port 0.06 s, 18552 KB, original 0.05 s, 16180 KB
- Emitted C++: 15422 bytes
