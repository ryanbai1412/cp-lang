# Friction Report: ioi06p4

## Dream

The port is the dream: a memoised `finish(l, r, at_r)` over arcs of the
`City` circle, with `cw`/`ccw` stepping round it.

## Gaps

- **Cyclic neighbours on a domain are spelled by hand.** `City.at(c + 1) ??
  City.first!` is the clockwise neighbour; a domain has no cyclic successor, so
  every circular problem writes these two helpers. Severity: low.
