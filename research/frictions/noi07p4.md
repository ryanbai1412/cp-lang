# Friction report: noi07p4

## Dream

```cp
var beads = SegTree(Run.join.opt(), [Slot] s => Some(Run.of(bead[s])), act: assign)
for a in arcs(i, j): beads.apply(a, Run.of(x))
```

## Changes

The v0.2 port read bare `Int` bounds, carried an `empty: Bool` flag in `Run` for the identity,
and computed slots with `% n` by hand. The rewrite types every bound, folds `?Run` through
`.opt()` so "no beads" is `None`, maps positions with `Slot.wrap`, and gathers the one or two
slot ranges of a segment in `arcs`, shared by paint and count.

## Remaining friction

- The stock `assign` action folds the tag `n` times on a custom op (O(log n)), so paint stays
  a hand `Action` that sets a range to one run in O(1).
- The C++ is over the 32 KB size target (W0201), as the v0.2 port's was.

## Verification

30 random cases agree with the v0.2 port and 30 with the accepted C++. Corpus tests pass; the
max case runs in 0.25 s.
