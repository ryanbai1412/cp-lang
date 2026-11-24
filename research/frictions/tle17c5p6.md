# tle17c5p6 — Circuits

## Memoised evaluation is a hand-cleared `[Wire]?Bool`
- **Where:** `port.cp`, `value` and the reset loop in `light`.
- **What felt wrong:** `out` is a memo over wires that must be forgotten for every `light(k)` call; `@memo` has no scope, so the cache is a global array cleared by hand.
- **General fix:** a scoped memo (`@memo(per: light)` or a `Memo<[Wire]Bool>` value passed in) that resets when the outer call returns.
- **Goal it hurts:** express.
- **Severity:** low.

## Dream

```cp
let n = read<1..=1e5>()
let m = read<1..=2e5>()
domain Wire = 1..=n + m
let gates = read<[m](a: Wire, b: Wire)>()
fn light(ones: 0..=n) -> Bool:
    @memo fn out(w: Wire) -> Bool =
        if w.label <= n: w.label <= ones else: not (out(gates[w.label - n - 1].a) and out(gates[w.label - n - 1].b))
    out(Wire.last!)
let dark = light(0)
if dark == light(n): print("0" * n)
else:
    let flip = (0..n).last_true(k => light(k) == dark)!
    print("1" * flip + "x" + "0" * (n - flip - 1))
```
