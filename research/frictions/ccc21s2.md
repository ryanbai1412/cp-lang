# Friction Report: ccc21s2

## A stroke's index space depends on its letter
- **Where:** `let kind: Char = read()` / `let at: Int = read()` / `Row(at)` or `Col(at)`.
- **What felt wrong:** `R r` and `C c` index different domains; the reader cannot type `at` by the
  letter, so the port reads an `Int` and converts it in each branch.
- **Goal it hurts:** hard-to-get-wrong.
- **Ideal code:** `let strokes: [k](R(Row) | C(Col)) = read()`: a tagged read whose tag picks the
  payload type.
- **General fix:** reading an enum from a leading tag token.
- **Severity:** low

## Dream

```cp
let (rows, cols, k): (1..=5e6, 1..=5e6, 1..=1e6) = read()
domain Row = 1..=rows
domain Col = 1..=cols
let strokes: [k](R(Row) | C(Col)) = read()
let r = Row.count(x => strokes.count(R(x)).odd)  // as parities: Row.count over a xor-fold
let c = Col.count(y => strokes.count(C(y)).odd)
print(r * (cols - c) + c * (rows - r))
```

## Measurements

- Tokens: port 161, original 403
- Max input (`gen.py --max`): port 0.01 s, 3792 KB, original 0.04 s, 3456 KB
- Emitted C++: 9042 bytes
