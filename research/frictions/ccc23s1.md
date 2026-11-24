# Friction Report: ccc23s1

## Pairs across two rows
- **Where:** `Col.count(c => c.ord % 2 == 0 and black[Row.first!, c] + black[Row.last!, c] == 2)`.
- **What felt wrong:** "the two rows' tiles at even columns" reads two named rows by
  `first!`/`last!`, and the 0/1 tiles are counted with `+ == 2` instead of `and`.
- **Goal it hurts:** brevity.
- **Ideal code:** `let black: [Row, Col]Bool = read(as: "01")` and
  `count(black[.., c].all() for c in Col.step_by(2))`.
- **General fix:** reading 0/1 tokens as `Bool`, and `step_by` on domains.
- **Severity:** low

## Dream

```cp
let n: 1..=2e5 = read()
domain Row = 0..2
domain Col = 0..n
let black: [Row, Col]Bool = read(as: "01")
let sideways = sum(black[r].adjacent.count((a, b) => a and b) for r in Row)
let across = count(black[.., c].all() for c in Col.step_by(2))
print(3 * black.count() - 2 * (sideways + across))
```

## Measurements

- Tokens: port 152, original 396
- Max input (`gen.py --max`): port 0.01 s, 17040 KB, original 0.01 s, 3840 KB
- Emitted C++: 16262 bytes
