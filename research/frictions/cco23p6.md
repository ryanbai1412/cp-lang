# cco23p6 — Triangle Collection (CCO 2023 P6)

Port: `corpus/problems/cco23p6/port.cp`. The old port already matched the editorial (a SegTree
of `Run(made, singles, pairs)` over lengths); the re-port drops its "as it builds today" note,
reads the root as `runs.all`, and writes `reach` and `refresh` as one-line fns. It gains a
generator and the 4 s judge limit.

## Frictions

- **A leaf derived from two tables is refreshed by hand.** A leaf is
  `join(Run(0, have[i] % 2, 0), Run(0, 0, ending[i]))`: its single and the pairs that reach it
  can match each other, so it can't be patched with `runs.update`, and both `count` calls
  `refresh` the two leaves they touch. A SegTree whose leaves are a fn of other tables
  (`SegTree.of(join, leaf)` that recomputes `leaf(i)` on a write to its inputs) would remove
  `refresh`.

## Dream

```cp
var runs = SegTree.of(join, i => join(Run(0, have[i] % 2, 0), Run(0, 0, ending[i])))
fn count(i: Len, sign: Int):
    pairs += sign * (have[i] / 2)
    ending[reach(i)] += sign * (have[i] / 2)
```

## Measurements

Max case (N = Q = 2e5, counts up to 1e9): port 0.11 s, accepted C++ 0.19 s, same output.
