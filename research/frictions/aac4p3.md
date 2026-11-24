# aac4p3 friction

The port's code is `dream.cp`, so there is no dream/port gap. One tooling gap affects both.

## The formatter splits a destructured loop variable over four lines
- **Where:** `port.cp`, the answer:
  ```
  sum(
      c * (c - 1) / 2 for (
          a,
          b,
      ) in directions for c in points.tally(by: p => b * p.y - a * p.x).values
  )
  ```
- **What felt wrong:** once the generator is too long, `cpl fmt` breaks the `(a, b)` pattern
  instead of the generator's `for` clauses, so a two-name pattern takes four lines.
- **Goal it hurts:** express.
- **Ideal code:**
  ```
  print(sum(
      c * (c - 1) / 2
      for (a, b) in directions
      for c in points.tally(by: p => b * p.y - a * p.x).values
  ))
  ```
- **General fix:** in `cpl fmt`, break a long generator or comprehension before each `for`/`if`
  clause, and keep short patterns on one line.
- **Severity:** low.
- **Owner:** Consumers (fmt). Repro: `cpl fmt corpus/problems/aac4p3/port.cp` with the ideal
  layout above.

## Measurements
- Tokens: port 229, original 357.
- `--max` (best of 10): port 0.061 s / 10 MB, accepted C++ 0.180 s / 9 MB. Same output.
- Emitted C++: 28,447 bytes.
