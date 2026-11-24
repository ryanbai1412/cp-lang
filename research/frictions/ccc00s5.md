# Friction Report: ccc00s5

## Two-decimal input read through floats
- **Where:** `let read_at: [n](x: F64, y: F64) = read()` then `round(s.x * SCALE)`.
- **What felt wrong:** the coordinates are exact decimals; the port reads binary floats and
  rounds them back to scaled integers, and float ranges can't carry the 0..=1000 bound as a type.
- **Goal it hurts:** types carry the proofs; common mistakes hard to write.
- **Ideal code:** `let sheep: [n](x: Fixed<2>, y: Fixed<2>) = read()` (exact decimals,
  bounded `0..=1000`), printed back with `print(p.x)`.
- **General fix:** a fixed-point decimal type in `std.math` that reads and prints exactly and
  converts to a scaled `Int` or a `Ratio`.
- **Severity:** medium

## Dream

```cp
let n: 1..=1000 = read()
let sheep: [n](x: Fixed<2, 0..=1000>, y: Fixed<2, 0..=1000>) = read()
for p in sheep:
    if sheep.any(q => q.x == p.x and q.y < p.y): continue
    fn cut(q) -> Ratio = (q.x^2 + q.y^2 - p.x^2 - p.y^2) / (2 * (q.x - p.x))
    let lo = max(0, max(cut(q) for q in sheep if q.x < p.x))
    let hi = min(1000, min(cut(q) for q in sheep if q.x > p.x))
    if lo <= hi: print("The sheep at ({p.x}, {p.y}) might be eaten.")
```

## Measurements

- Tokens: port 285, original 652
- Max input (`gen.py --max`): port 0.07 s, 3456 KB, original 0.00 s, 3840 KB
- Emitted C++: 18113 bytes
