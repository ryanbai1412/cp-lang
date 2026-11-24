# Friction Report: etsr23r3p1

## Dream

The port is the dream: a `SegTree(act:)` over the compressed starts whose tag is
the affine map `v -> mul * v + m * a + b`, with a clamp being the map with
`mul = m = 0`, and two binary searches on point reads per operation.

## Gaps

- The tags' coefficients have to be `U64` to wrap: a start that is never
  clamped can multiply its coefficient past `Int` while its value stays in
  `lo..=hi`. The value still comes out exact mod 2^64, but nothing in the
  types says so; `Int(u)` traps above 2^63, so a value below 0 before its clamp
  is read back through a 2^31 shift. A signed wrapping integer (or `I64.wrap(u)`)
  would remove the reasoning.
- An act-only tree still needs an `over:` monoid; `Monoid(zero, (x, _) => x)`
  is a dummy that is never folded.
- The statement is not public, and the accepted reference is right only on
  non-negative values (a negative start can leave `lo..=hi` in it), so the
  bounds are 0..2^31.
