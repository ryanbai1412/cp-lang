# Friction report: ioi22p4

## Dream

```cp
type Z = Mod<1_000_002_022>
@grader fn init(tree: Rooted<Node>, lit: [Source]Bool)
let ways = tree.fold_up(u => Z(tree.children(u).len) * prod(ways[v] for v in tree.children(u)))
let worth = tree.fold_down(Z(1), (w, u, v) => w * prod_except(ways, tree.children(u), v))
var sources = SegTree(Flip.op(+), [Source] s => Flip.of(lit[s], worth[s], Z(0)), act: flip)
@grader fn count_ways(l: Source, r: Source) -> Z:
    sources.apply(l..=r, true)
    sources.all.now
```

## Stock `flip` is not wired up

`act: flip` fails: `Action.of` has branches for `add`, `assign` and `affine` only. The port
uses `flip_action<Z>(+)` with a `use std.lazy.{flip_action}`. Reported to the stdlib owner.

## `Flip.op(+)` is a std::function

`Flip<Z>.op(+)` stores a capturing closure in `Monoid.op`, which `cpl explain` lowers to a
`std::function`; the port writes `join` and `Monoid(Flip(Z(0), Z(0)), join)` instead.
Reported with the above.

## Products of all but one child

The modulus is not prime, so "all children but one" is a prefix times a suffix product, with
`after` and `before` written by hand. A std `prod_except` (or `exclusive_scan` both ways)
would name it.

## Tree folds over a parent array

Gates are `Node.first!..Node(n)` walked backwards for the upward pass and forwards for the
downward one, relying on `P[i] < i`. `Rooted` from a parent array with `fold_up` /
`fold_down` would say which direction each pass goes.

## The grader lives in the solution

`init` / `count_ways` are grader functions; the port reads their arguments from stdin.

## Verification

30 random cases agree with the v0.2 port and 30 with the accepted C++ (`orig.cpp`, the AC
grader functions plus a stdin `main`). The max case (n = m = q = 1e5) is byte-identical to
the C++ and runs in 0.14 s. `cpl explain`: 0 std::function.
