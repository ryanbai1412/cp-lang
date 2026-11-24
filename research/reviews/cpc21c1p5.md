# cpc21c1p5

Score: 3/5
Dream: 4/5

The port follows the header's argument: four monotone searches pin down a rectangle, then its residual count gives the multiplicity (L26–30). The friction's `## Dream` is the port with `left`'s query inlined, and the port differs only by the reported `ask` workaround; one unreported typing gap holds it at 3.

## Checklist failures
- idiomatic-code/R5, unreported: L13 states `const SIDE = 1e9`, but L15 uses `type Box = (x1: Int, x2: Int, y1: Int, y2: Int)` and L19 takes four `Int` coordinates; the coordinate range belongs in those types.

## Owed rewrites
- Give the coordinates a range type from `SIDE` and use it in `Box` and `left` (L15, L19).

## Language gaps
- `ask` only works as a `let` initializer (E0501 in the friction); L20's separate `let inside` is the workaround.

## Dream notes
The dream keeps the named monotone searches and only inlines the query into `left`; it inherits the untyped `Int` coordinates, which keeps it at 4.
