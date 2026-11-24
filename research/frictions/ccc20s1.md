# Friction Report: ccc20s1

## The largest Ratio cannot be printed
- **Where:** `speeds.max(by: s => Ratio(s.dx, s.dt))` then `fixed(top.dx, top.dt, 5)`.
- **What felt wrong:** `Ratio`'s numerator and denominator are private, so the port keeps the
  pair beside the key instead of taking the max of the ratios and printing it.
- **Goal it hurts:** brevity; one spelling per idea.
- **Ideal code:** `print(fixed(seen.sorted(by: o => o.t).adjacent.map((a, b) => Ratio(abs(b.x - a.x), b.t - a.t)).max()!, 5))`.
- **General fix:** a `fixed(r: Ratio, digits)` overload (or a public `num`/`den` reader).
- **Severity:** low

## Dream

```cp
let n: 2..=1e5 = read()
let seen: [n](t: -1e9..=1e9, x: -1e9..=1e9) = read()
print(fixed(seen.sorted(by: o => o.t).adjacent.map((a, b) => Ratio(abs(b.x - a.x), b.t - a.t)).max()!, 5))
```

## Measurements

- Tokens: port 131, original 389
- Max input (`gen.py --max`): port 0.02 s, 10920 KB, original 0.01 s, 4480 KB
- Emitted C++: 15216 bytes
