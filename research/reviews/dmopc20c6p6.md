# dmopc20c6p6

Score: 3/5
Dream: none (no dream.cp; the ideal is in the friction report)

Scored cold against the checklist and the port.

An interactive measurement: bucket capacities are learned by dumping unmeasured buckets into the current one through `wire.ask`, tracking the largest so far.

## Checklist failures

- `cap[largest!]!` unwraps twice in one comparison inside `measured`.
- `unmeasured_from(b)` walks forward with `while let x = at and cap[x] != None: at = x.next`, a hand search for the next unmeasured bucket.
- The main loop is a `loop:` with several `break`s and a `var gave: ?Int` that carries state between iterations; the protocol's phases have no names.
- `empty`, `fill` and `dump` print raw protocol strings; only `dump` goes through `wire.ask`.
