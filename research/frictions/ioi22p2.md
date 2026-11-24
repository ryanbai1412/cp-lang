# Friction report: ioi22p2

## Dream

```cp
enum Bag: A | B
enum Act: Smaller(Bag) | Write(State)
let tree = SplitTree(1..=5588, fanout: [3, 3, 3, 3, 3, 2, 2, 1])    // endpoints kept per node
domain State = tree.edges_by_depth                                  // (depth, child), 21 of them
@grader fn devise_strategy(n: 2..=5000) -> [State](opens: Bag, act: [1..=n]Act)
```

## The grader's integer protocol

The grader wants `-1` / `-2` for "A / B is smaller" and a state number otherwise. The port
keeps `Act` and `Bag` typed and converts in `code`; a grader-function entry with a declared
output encoding would make `code` and the leading `Int(bag == Bag.B)` disappear.

## States as a domain

States are numbered by hand with `first_state(t) = 1 + prefix of fanout`, and the loop
rebuilds `(t, c)` from nested ranges. A domain built from the per-depth fanout (a ragged
product, `(t: 0..=depth, c: 0..fanout[t - 1])`) would number them and index the rows.

## Nested enum patterns

`match a: Act.Smaller(Bag.A) => ...` is E0314 (no arm for `Smaller`): nested constructor
patterns don't count toward exhaustiveness, so `code` matches `Act.Smaller(b)` and tests `b`.

## The interval tree is hand-built

`size` and `path` hand-code a fixed-fanout interval split with kept endpoints. It is
specific to this task, so it stays in the port.

## Verification

30 random cases match the v0.2 port byte for byte, and 30 pass the checker against the
accepted C++ (`orig.cpp`, the AC grader function plus a stdin `main`). The checker tries
every pair for n <= 200, otherwise the near pairs plus 100000 random ones. The max case
(n = 5000) passes the checker. `scripts/corpus.py verify` passes, dream included.
