# noi08p5

Score: 3/5
Dream: n/a

Scored cold against the checklist, the dream and the port.

The port reads each event into an `Event` enum, keeps every cloud as a point (phase, width) in a `Fenwick2D`, and answers each question as rectangle counts over two unrolled planes. It agrees with the v0.2 port and the accepted C++ on generated cases.

## Checklist failures

- `count(ox, oy, xl, xr, yl, yr)` takes six bare `Int`s and clamps each rectangle by hand (`max(xl - s, 0)`, `min(xr - s, p - 1)`). A rectangle type clipped to the Fenwick's domains would carry the bounds.
- `phase` and `width` are two parallel `[Colour]Int` arrays written while reading and read again on removal.
- The event kind is read as `1..=3` and dispatched with `match kind` and a `_ =>` arm that stands for kind 3.
- The two planes are two passes (`for plane in 0..2`) that clear and refill the same Fenwick.

## Gaps that need the language

- Two Fenwick trees don't fit, and events read by hand instead of as a tagged enum: research/frictions/noi08p5.md.
