# Friction Report: ccc19s5

## A triangle has no domain
- **Where:** `var best: [0..n, 0..n]Int` with `for j in 0..=i`.
- **What felt wrong:** row and column of a triangle are distinct index spaces, but `Row`/`Col`
  domains make `best[i + d, j + d]` an `Int` index (E0301), so the port keeps plain ranges and
  half of the square table is never used.
- **Goal it hurts:** types carry the proofs; memory.
- **Ideal code:** `var best: Triangle<Row, Int>` with `best[i + d, j + d]` checked as a domain
  offset (`i.offset(d)?`).
- **General fix:** domain offsets that return a domain value (`Row + Int -> ?Row` or a checked
  `i + d` when the range proves it), and a triangular table in `std.grid`.
- **Severity:** medium

## Dream

```cp
let n: 1..=3000 = read()
let k: 1..=n = read()
domain Row = 0..n
var best = Triangle<Row, Int>.read()
for (side, grown) in iterate(1, s => if s < k: Some(min(k, (s * 3 + 1) / 2)) else: None).adjacent:
    let d = grown - side
    for (i, j) in best.apexes(side: grown): best[i, j] max= max(best[i + d, j], best[i + d, j + d])
print(sum(best[i, j] for (i, j) in best.apexes(side: k)))
```

## Measurements

- Tokens: port 183, original 1000
- Max input (`gen.py --max`): port 0.15 s, 77240 KB, original 0.50 s, 172032 KB
- Emitted C++: 9439 bytes
