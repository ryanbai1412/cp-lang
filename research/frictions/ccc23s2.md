# Friction Report: ccc23s2

## Palindromic expansion is index arithmetic
- **Where:** `for j in 0..=min(centre, n - 1 - centre - off): cost += abs(h[centre - j] - h[centre + j + off])`.
- **What felt wrong:** "walk outward from a centre while both ends exist" needs the bound
  computed by hand, and the heights are indexed by plain `Int` because a domain offset is an
  `Int` (E0301), so the walk is not proven in bounds.
- **Goal it hurts:** types carry the proofs.
- **Ideal code:** `for (a, b) in h.outward(from: centre, gap: off): cost += abs(a - b)`.
- **General fix:** an `outward(from:, gap:)` pair walk in `std.seq`, or checked domain offsets.
- **Severity:** low

## Dream

```cp
let n: 1..=5000 = read()
let h: [n]0..=1e5 = read()
domain Len = 1..=n
var least: [Len]?Int = None
for centre in 0..n, off in 0..=1:
    for (j, cost) in h.outward(from: centre, gap: off).map((a, b) => abs(a - b)).scan(+).enumerate():
        least[Len(2 * j + 1 + off)] min= cost
print(least.map(x => x!))
```

## Measurements

- Tokens: port 137, original 492
- Max input (`gen.py --max`): port 0.01 s, 3456 KB, original 0.00 s, 3584 KB
- Emitted C++: 10333 bytes
