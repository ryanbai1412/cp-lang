# Friction Report: ccc22s4

## Counting a domain-valued array into `[D]Int`
- **Where:** `var at: [Spot]Int = 0` / `for p in points: at[p] += 1`.
- **What felt wrong:** `tally` returns a `Counter` map; a dense count over a domain has no
  one-call spelling.
- **Goal it hurts:** brevity.
- **Ideal code:** `let at = points.tally(over: Spot)` returning `[Spot]Int`.
- **General fix:** `tally(over: D)` on sequences of a domain type.
- **Severity:** low

## Half-circle sliding window by hand
- **Where:** `ahead += at[Spot.wrap(p.ord + half + 1)] - at[Spot.wrap(p.ord + 1)]`.
- **What felt wrong:** `at.ring.windows(half)` walks the same windows but yields slices, and a
  slice sum per window is O(c / 2); the port keeps the running sum itself.
- **Goal it hurts:** brevity.
- **Ideal code:** `at.ring.window_sums(half, from: 1)`.
- **General fix:** running folds over ring windows for invertible ops.
- **Severity:** low

## Dream

```cp
let (n, c): (3..=1e6, 3..=1e6) = read()
domain Spot = 0..c
let at = read<[n]Spot>().tally(over: Spot)
let ahead = at.ring.window_sums(c / 2, from: 1)
let bad = sum(choose(at[p], 3) + choose(at[p], 2) * ahead[p] + at[p] * choose(ahead[p], 2) for p in Spot)
let twice = if c.even: sum(choose(at[p], 2) * at[q] + at[p] * choose(at[q], 2)
    for p in Spot.take(c / 2) let q = p + c / 2) else: 0
print(choose(n, 3) - bad + twice)
```

## Measurements

- Tokens: port 256, original 510
- Max input (`gen.py --max`): port 0.03 s, 15952 KB, original 0.08 s, 11392 KB
- Emitted C++: 11193 bytes
