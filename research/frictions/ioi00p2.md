# Friction Report: ioi00p2

## Dream

```cp
domain Slot = 0..n
var park: [Slot]Kind = read()
let want = park.sorted()
for round in rounds: print(round.len, round.flat)   // (from, to) pairs, flattened
```

## Gaps

- **Slots stay `Int`.** The chain walk does slot arithmetic (`cursor[t] += 1`,
  `from + 1` for 1-based output), so a `Slot` domain would need `.next` and
  `.label` at every step. Severity: low.
- **Moves are a flat `List<Int>` of from/to pairs.** A `List<(Int, Int)>`
  would read better but needs flattening to print. Severity: low.
