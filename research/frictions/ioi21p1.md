# Friction report: ioi21p1

## Dream

```cp
@grader fn distribute_candies(cap: [Box]Int, days: [Day](l: Box, r: Box, v: Int)) -> [Box]Int:
    var walk = SegTree<Day, Walk>(Walk.then)
    for b in Box sweeping days by (l, r):
        let from = walk.left_while(Day.cuts.last!, w => w.span < cap[b])
        let tail = walk.fold(from..)
        if let p = from.prev and days[p].v > 0: cap[b] - tail.below_top else: tail.above_low
```

## The grader lives in the solution

`distribute_candies` is a grader function; the port reads its arguments from stdin.

## Range events by box

`opens` and `closes` bucket days by their first and last box by hand. A sweep helper that
walks a domain while switching interval events on and off (`for b in Box sweeping days`)
would name the pattern; it recurs across offline range problems.

## The day before a cut

`Day.at(Int(from) - 1)` reads the position just left of a cut through integers. A
`cut.prev: ?D` (and `cut.next`) would say it directly.

## A monoid from a struct

`SegTree<Day, Walk>(Monoid(Walk(0, 0, 0), then))` names the identity and the op apart from
`Walk`. An `impl Monoid for Walk` (or `Walk.then` carrying its identity) would let the tree
take the type alone. `fold(from..Day.cuts.last!)` would read as `fold(from..)` with an
open-ended cut range.

## Verification

30 random cases agree with the v0.2 port and 30 with the accepted C++ (`orig.cpp`, the AC
grader function plus a stdin `main`); the max case (n = q = 2e5) is byte-identical to both
and runs in 0.11 s.
